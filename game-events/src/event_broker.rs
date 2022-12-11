use crate::{
    actions::compound_events::GameAction,
    game_events::{EventConsumer, GameEventObject},
};

pub struct EventBroker {
    sender_id: String,
    start_of_turn: usize,
    past_events: Vec<GameAction>,
    pub(crate) subscribers: Vec<Box<dyn EventConsumer>>,
}

impl EventBroker {
    pub fn new(sender_id: String) -> Self {
        EventBroker {
            sender_id,
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
            let event_object = GameEventObject::new(event.anti_event(), &self.sender_id);
            self.handle_event(&event_object);
        }
    }

    pub fn handle_new_event(&mut self, event: &GameAction) {
        self.past_events.push(event.clone());

        let event_object = GameEventObject::new(event.clone(), &self.sender_id);
        self.handle_event(&event_object);
    }

    pub fn handle_remote_event(&mut self, event: &GameEventObject) {
        self.handle_event(&event);
    }

    pub fn get_past_events(&self) -> &Vec<GameAction> {
        &self.past_events
    }
}

impl EventConsumer for EventBroker {
    fn handle_event(&mut self, event: &GameEventObject) {
        self.subscribers
            .iter_mut()
            .for_each(|s| (*s).handle_event(event));

        if let GameAction::FinishTurn(_) = event.event {
            self.start_of_turn = self.past_events.len();
        }
    }
}
