use std::{cell::RefCell, rc::Rc};

use game_events::{
    actions::compound_events::GameAction,
};
use crate::game_controller::GameCommand;
use crate::game_events::{Event, EventConsumer, GameEventObject, PlayerAction};

use crate::multiplayer_connector::MultiplayerConector;

pub struct EventBroker {
    start_of_turn: usize,
    past_events: Vec<GameAction>,
    pub(crate) subscribers: Vec<Box<dyn EventConsumer>>,
}

impl EventBroker {
    pub fn new() -> Self {
        EventBroker {
            start_of_turn: 0,
            past_events: vec![],
            subscribers: vec![],
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
            let anti_event = event.anti_event();
            self.handle_event_internal(&anti_event);
        }
    }

    pub fn handle_new_event(&mut self, event: &GameAction) {
        self.past_events.push(event.clone());
        self.handle_event_internal(&event);
    }

    pub fn get_past_events(&self) -> &Vec<GameAction> {
        &self.past_events
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
