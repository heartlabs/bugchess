use game_logic::piece::{Exhaustion, Piece, PieceKind};
use game_logic::board::{Point2};
use crate::atomic_events::{AtomicEvent};
use nanoserde::{DeBin, SerBin};
use std::fmt::Debug;
use crate::actions::attack::{AttackBuilder, AttackCompoundEvent};
use crate::actions::finish_turn::{FinishTurnBuilder, FinishTurnCompoundEvent};
use crate::actions::moving::{MoveBuilder, MoveCompoundEvent};
use crate::actions::place::{PlaceBuilder, PlaceCompoundEvent};
use crate::actions::undo::UndoCompoundEvent;

pub trait CompoundEvent: Debug {
    fn get_events(&self) -> Vec<AtomicEvent>;

    fn push_event(&mut self, event: AtomicEvent); // TODO remove
                                                  //    fn get_event_type(&self) -> &CompoundEventType;

    fn flush(&mut self) -> Vec<AtomicEvent>;
}

#[derive(Debug, Clone, SerBin, DeBin)]
pub enum GameAction {
    Attack(AttackCompoundEvent),
    Place(PlaceCompoundEvent),
    Move(MoveCompoundEvent),
    Undo(UndoCompoundEvent),
    FinishTurn(FinishTurnCompoundEvent),
}

impl GameAction {
    pub fn attack(piece_kind: PieceKind) -> GameAction {
        AttackBuilder::new(piece_kind).build()
    }

    pub fn place() -> GameAction {
        PlaceBuilder::new().build()
    }

    pub fn moving() -> GameAction {
        MoveBuilder::new().build()
    }

    pub fn undo(undone: Box<GameAction>) -> GameAction {
        GameAction::Undo(UndoCompoundEvent {
            events: vec![],
            undone,
            was_flushed: false,
        })
    }

    pub fn finish_turn() -> FinishTurnBuilder {
        FinishTurnBuilder::new()
    }

    pub fn get_compound_event(&self) -> Box<&dyn CompoundEvent> {
        match self {
            GameAction::Attack(e) => Box::new(e),
            GameAction::Place(e) => Box::new(e),
            GameAction::Move(e) => Box::new(e),
            GameAction::Undo(e) => Box::new(e),
            GameAction::FinishTurn(e) => Box::new(e),
        }
    }

    pub fn get_compound_event_mut(&mut self) -> Box<&mut dyn CompoundEvent> {
        match self {
            GameAction::Attack(e) => Box::new(e),
            GameAction::Place(e) => Box::new(e),
            GameAction::Move(e) => Box::new(e),
            GameAction::Undo(e) => Box::new(e),
            GameAction::FinishTurn(e) => Box::new(e),
        }
    }

    pub fn anti_event(&self) -> GameAction {
        GameAction::Undo(UndoCompoundEvent {
            events: self
                .get_compound_event()
                .get_events()
                .iter()
                .map(|e| e.anti_event())
                .rev()
                .collect(),
            undone: Box::new(self.clone()),
            was_flushed: false,
        })
    }
}
