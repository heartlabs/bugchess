use std::fmt::Display;

use crate::{
    actions::compound_events::{CompoundEvent, GameAction},
    atomic_events::AtomicEvent,
};
use nanoserde::{DeJson, SerJson};

#[derive(Debug, Clone, SerJson, DeJson)]
pub struct UndoCompoundEvent {
    pub events: Vec<AtomicEvent>,
    pub undone: Box<GameAction>,
    pub was_flushed: bool,
}

pub struct UndoBuilder {
    event: UndoCompoundEvent,
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
            },
        }
    }
}

impl CompoundEvent for UndoCompoundEvent {
    fn get_events(&self) -> Vec<AtomicEvent> {
        self.events.clone()
    }
}

impl Display for UndoCompoundEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Undo {}", self.undone)
    }
}
