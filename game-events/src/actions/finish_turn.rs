use crate::{actions::compound_events::CompoundEvent, atomic_events::AtomicEvent};
use game_model::{
    board::Point2,
    piece::{Exhaustion, Piece},
};
use nanoserde::{DeBin, SerBin};

use super::compound_events::GameAction;

#[derive(Debug, Clone, SerBin, DeBin)]
pub struct FinishTurnCompoundEvent {
    events: Vec<AtomicEvent>,
    was_flushed: bool,
}

impl CompoundEvent for FinishTurnCompoundEvent {
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

pub struct FinishTurnBuilder {
    event: FinishTurnCompoundEvent,
}

impl FinishTurnBuilder {
    pub(crate) fn new() -> FinishTurnBuilder {
        FinishTurnBuilder {
            event: FinishTurnCompoundEvent {
                events: vec![AtomicEvent::NextTurn],
                was_flushed: false,
            },
        }
    }
}

impl FinishTurnBuilder {
    pub fn build(self) -> GameAction {
        GameAction::FinishTurn(self.event)
    }

    pub fn place_piece(&mut self, point: Point2, piece: Piece) -> &mut Self {
        self.event.events.push(AtomicEvent::Place(point, piece));

        self
    }

    pub fn add_unused_piece(&mut self, team: usize) -> &mut Self {
        self.event.events.push(AtomicEvent::AddUnusedPiece(team));

        self
    }

    pub fn change_exhaustion(&mut self, from: Exhaustion, to: Exhaustion, at: Point2) -> &mut Self {
        self.event
            .events
            .push(AtomicEvent::ChangeExhaustion(from, to, at));

        self
    }
}
