use std::fmt::Display;

use crate::{
    actions::compound_events::{CompoundEvent, GameAction},
    atomic_events::AtomicEvent,
};
use game_model::{
    board::Point2,
    piece::{Exhaustion, Piece},
};
use nanoserde::{DeBin, SerBin};

#[derive(Debug, Clone, SerBin, DeBin)]
pub struct FinishTurnCompoundEvent {
    events: Vec<AtomicEvent>,
    was_flushed: bool,
}

impl CompoundEvent for FinishTurnCompoundEvent {
    fn get_events(&self) -> Vec<AtomicEvent> {
        self.events.clone()
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

impl Display for FinishTurnCompoundEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FinishTurn")
    }
}