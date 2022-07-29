use crate::events::game_events::{EventConsumer, GameEventObject};
use crate::events::compound_events::GameAction;

pub struct EventBroker {
    sender_id: String,
    past_events: Vec<GameAction>,
    pub(crate) subscribers: Vec<Box<dyn EventConsumer>>,
}

impl EventBroker {
    pub(crate) fn new(sender_id: String) -> Self {
        EventBroker {
            sender_id,
            past_events: vec![],
            subscribers: vec![],
        }
    }

    pub(crate) fn subscribe_committed(&mut self, subscriber: Box<dyn EventConsumer>) {
        self.subscribers.push(subscriber);
    }

    pub fn undo(&mut self) {
        if let Some(event) = self.past_events.pop() {
            let event_object = GameEventObject::new(event.anti_event(), &self.sender_id);
            self.handle_event(&event_object);
        }
    }

    pub fn delete_history(&mut self) {
        self.past_events.clear();
    }

    pub fn handle_new_event(&mut self, event: &GameAction) {
        self.past_events.push(event.clone());

        let event_object = GameEventObject::new(event.clone(), &self.sender_id);
        self.handle_event(&event_object);
    }

    pub fn handle_remote_event(&mut self, event: &GameEventObject) {
        self.handle_event(&event);
    }
}

impl EventConsumer for EventBroker {
    fn handle_event(&mut self, event: &GameEventObject) {
        self.subscribers
            .iter_mut()
            .for_each(|s| (*s).handle_event(event));

        if let GameAction::FinishTurn(_) = event.event {
            self.delete_history();
        }
    }
}
