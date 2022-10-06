use crate::{
    actions::{compound_events::CompoundEvent, merge::MergeCompoundEvent},
    atomic_events::AtomicEvent,
};
use nanoserde::{DeBin, SerBin};

use super::compound_events::GameAction;

#[derive(Debug, Clone, SerBin, DeBin)]
pub struct PlaceCompoundEvent {
    events: Vec<AtomicEvent>,
    merge_events: MergeCompoundEvent,
    was_flushed: bool,
}

pub struct PlaceBuilder {
    event: PlaceCompoundEvent,
}

impl PlaceBuilder {
    pub fn build(self) -> GameAction {
        GameAction::Place(self.event)
    }
}

impl PlaceBuilder {
    pub(crate) fn new() -> Self {
        PlaceBuilder {
            event: PlaceCompoundEvent {
                events: vec![],
                merge_events: MergeCompoundEvent::new(),
                was_flushed: false,
            },
        }
    }
}

impl CompoundEvent for PlaceCompoundEvent {
    fn get_events(&self) -> Vec<AtomicEvent> {
        let mut all_events: Vec<AtomicEvent> = vec![];
        all_events.extend(&self.events);
        all_events.extend(&self.merge_events.events);
        all_events
    }

    fn push_event(&mut self, event: AtomicEvent) {
        if self.was_flushed {
            self.merge_events.push_event(event);
        } else {
            self.events.push(event);
        }
    }

    fn flush(&mut self) -> Vec<AtomicEvent> {
        if self.was_flushed {
            return self.merge_events.flush();
        }

        self.was_flushed = true;

        self.get_events()
    }
}
