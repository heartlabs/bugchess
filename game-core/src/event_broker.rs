use std::{rc::Rc, cell::RefCell};

use game_events::{
    actions::compound_events::GameAction,
    game_events::{EventConsumer, GameEventObject, PlayerAction, Event},
};

use crate::multiplayer_connector::MultiplayerConector;

pub struct EventBroker {
    sender_id: String,
    start_of_turn: usize,
    past_events: Vec<GameAction>,
    pub(crate) subscribers: Vec<Box<dyn EventConsumer>>,
    pub multiplayer_connector: Option<Rc<RefCell<MultiplayerConector>>>
}

impl EventBroker {
    pub fn new(sender_id: String) -> Self {
        EventBroker {
            sender_id,
            start_of_turn: 0,
            past_events: vec![],
            subscribers: vec![],
            multiplayer_connector: None,
        }
    }

    pub fn subscribe(&mut self, subscriber: Box<dyn EventConsumer>) {
        self.subscribers.push(subscriber);
    }

    pub fn undo(&mut self) {
        if self.past_events.len() <= self.start_of_turn {
            return;
        }

        if let Some(event) = self.past_events.pop() {
            self.handle_event(&event.anti_event());
        }
    }

    pub fn handle_new_event(&mut self, event: &GameAction) {
        self.past_events.push(event.clone());

        self.handle_event(&event);
    }

    pub fn get_past_events(&self) -> &Vec<GameAction> {
        &self.past_events
    }

    pub fn handle_remote_event(&mut self, event_object: &GameEventObject) {

        if let Event::GameAction(game_action) = &event_object.event {
            self.handle_event_internal(game_action);
        }

        match &event_object.event {
            Event::PlayerAction(PlayerAction::Connect(_)) => {
               // self.handle_event(Event::PlayerAction(PlayerAction::SendGame(self.past_events.clone())));
            },
            Event::PlayerAction(PlayerAction::SendGame(game_events)) => {
                if self.past_events.is_empty() {
                    game_events.iter().for_each(|event| self.handle_event(event))
                }
            },
            _ => {}
        }
    }

    fn handle_event_internal(&mut self, event: &GameAction) {
        self.subscribers
            .iter_mut()
            .for_each(|s| (*s).handle_event(event));

        if let GameAction::FinishTurn(_) = event {
            self.start_of_turn = self.past_events.len();
        }
    }
}

impl EventConsumer for EventBroker {
    fn handle_event(&mut self, event: &GameAction) {
        self.handle_event_internal(event);
        if let Some(multiplayer_connector) = self.multiplayer_connector.as_mut() {
            (*multiplayer_connector)
                .borrow_mut()
                .handle_event(event);
        }
    }
}
