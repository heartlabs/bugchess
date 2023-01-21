use crate::{
    actions::{
        attack::AttackCompoundEvent,
        finish_turn::{FinishTurnBuilder, FinishTurnCompoundEvent},
        merge::{MergeBuilder, MergeCompoundEvent},
        moving::MoveCompoundEvent,
        place::{PlaceBuilder, PlaceCompoundEvent},
        undo::{UndoBuilder, UndoCompoundEvent},
    },
    atomic_events::AtomicEvent,
};
use game_model::{Point2, piece::Piece};
use nanoserde::{DeBin, SerBin};
use std::fmt::{Debug, Display};

pub trait CompoundEventBuilder {
    fn build_with_merge_event(self: Box<Self>, merge_event: MergeCompoundEvent) -> GameAction;

    fn build(self) -> GameAction;

    fn flush(self: Box<Self>, consumer: &mut dyn FnMut(&AtomicEvent)) -> FlushResult;
}

pub enum FlushResult {
    Merge(MergeBuilder),
    Build(GameAction),
}

pub trait CompoundEvent: Debug {
    fn get_events(&self) -> Vec<AtomicEvent>;
}

#[derive(Debug, Clone, SerBin, DeBin)]
pub enum GameAction {
    Attack(AttackCompoundEvent),
    Place(PlaceCompoundEvent),
    Move(MoveCompoundEvent),
    Undo(UndoCompoundEvent),
    FinishTurn(FinishTurnCompoundEvent),
}

impl Display for GameAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameAction::Attack(a) => Display::fmt(&a, f),
            GameAction::Place(a) => Display::fmt(&a, f),
            GameAction::Move(a) => Display::fmt(&a, f),
            GameAction::Undo(a) => Display::fmt(&a, f),
            GameAction::FinishTurn(a) => Display::fmt(&a, f),
        }
    }
}

impl GameAction {
    pub fn place(at: Point2, piece: Piece, team_id: usize) -> PlaceBuilder {
        PlaceBuilder::new(at, piece, team_id)
    }

    pub fn undo(undone: Box<GameAction>) -> GameAction {
        UndoBuilder::new(undone).build()
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
