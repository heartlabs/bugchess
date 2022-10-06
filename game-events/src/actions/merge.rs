use nanoserde::{DeBin, SerBin};
use crate::atomic_events::AtomicEvent;
use crate::actions::compound_events::CompoundEvent;

#[derive(Debug, Clone, SerBin, DeBin)]
pub struct MergeCompoundEvent {
    pub events: Vec<AtomicEvent>,
    was_flushed: bool,
}

impl MergeCompoundEvent {
    pub fn new() -> Self {
        MergeCompoundEvent {
            events: vec![],
            was_flushed: false,
        }
    }
}

impl CompoundEvent for MergeCompoundEvent {
    fn get_events(&self) -> Vec<AtomicEvent> {
        self.events.clone()
    }

    fn push_event(&mut self, event: AtomicEvent) {
        assert!(
            !self.was_flushed,
            "Cannot push event after already being flushed"
        );
        self.events.push(event);
    }

    fn flush(&mut self) -> Vec<AtomicEvent> {
        if self.was_flushed {
            return vec![];
        }

        self.was_flushed = true;

        self.get_events()
    }
}
