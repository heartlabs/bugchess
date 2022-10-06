use nanoserde::{DeBin, SerBin};
use crate::atomic_events::{AtomicEvent};
use crate::actions::compound_events::CompoundEvent;

use super::compound_events::GameAction;

#[derive(Debug, Clone, SerBin, DeBin)]
pub struct UndoCompoundEvent {
    pub events: Vec<AtomicEvent>,
    pub undone: Box<GameAction>,
    pub was_flushed: bool,
}


pub struct UndoBuilder {
    event: UndoCompoundEvent
}

impl UndoBuilder {
    pub fn build(self) -> GameAction {
        GameAction::Undo(self.event)
    }
}

impl UndoBuilder {
    pub(crate) fn new(undone: Box<GameAction>) -> Self {
        UndoBuilder {
            event: UndoCompoundEvent {
                events: vec![],
                undone,
                was_flushed: false,
            }
        }
    }
}

impl CompoundEvent for UndoCompoundEvent {
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
