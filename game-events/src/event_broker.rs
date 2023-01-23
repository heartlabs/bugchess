use crate::actions::compound_events::GameAction;

pub trait EventConsumer {
    fn handle_event(&mut self, event: &GameAction);
}

pub struct EventBroker {
    past_events: Vec<GameAction>,
    subscribers: Vec<Box<dyn EventConsumer>>,
}

impl EventBroker {
    pub fn new() -> Self {
        EventBroker {
            past_events: vec![],
            subscribers: vec![],
        }
    }

    pub fn subscribe(&mut self, subscriber: Box<dyn EventConsumer>) {
        self.subscribers.push(subscriber);
    }

    pub fn undo(&mut self) {
        if let Some(event) = self.past_events.pop() {
            let anti_event = event.anti_event();
            self.handle_event_internal(&anti_event);
        }
    }

    pub fn handle_new_event(&mut self, event: &GameAction) {
        self.past_events.push(event.clone());
        self.handle_event_internal(&event);
    }

    fn handle_event_internal(&mut self, event: &GameAction) {
        self.subscribers
            .iter_mut()
            .for_each(|s| (*s).handle_event(event));
    }
}
