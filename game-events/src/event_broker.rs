use crate::actions::compound_events::GameAction;

pub trait EventConsumer {
    fn handle_event(&mut self, event: &GameAction);
}

pub struct EventBroker {
    subscribers: Vec<Box<dyn EventConsumer>>,
}

impl Default for EventBroker {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBroker {
    pub fn new() -> Self {
        EventBroker {
            subscribers: vec![],
        }
    }

    pub fn subscribe(&mut self, subscriber: Box<dyn EventConsumer>) {
        self.subscribers.push(subscriber);
    }

    pub fn dispatch(&mut self, event: &GameAction) {
        self.subscribers
            .iter_mut()
            .for_each(|s| (*s).handle_event(event));
    }
}
