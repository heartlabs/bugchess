use crate::atomic_events::AtomicEvent::*;
use game_model::{
    piece::{EffectKind, Exhaustion, Piece},
    Point2,
};
use nanoserde::{DeBin, SerBin};

#[derive(Debug, Copy, Clone, SerBin, DeBin)]
pub enum AtomicEvent {
    Place(Point2, Piece),
    Remove(Point2, Piece),
    AddUnusedPiece(usize),
    RemoveUnusedPiece(usize),
    ChangeExhaustion(Exhaustion, Exhaustion, Point2), // From, To, At
    AddEffect(EffectKind, Point2),
    RemoveEffect(EffectKind, Point2),
    NextTurn,
}

impl AtomicEvent {
    pub fn anti_event(&self) -> AtomicEvent {
        match self {
            //    Move(from, to) => Move(*to, *from),
            Place(at, piece) => Remove(*at, *piece),
            Remove(at, piece) => Place(*at, *piece),
            AddUnusedPiece(team_id) => RemoveUnusedPiece(*team_id),
            RemoveUnusedPiece(team_id) => AddUnusedPiece(*team_id),
            ChangeExhaustion(from, to, point) => ChangeExhaustion(*to, *from, *point),
            AddEffect(kind, at) => RemoveEffect(*kind, *at),
            RemoveEffect(kind, at) => AddEffect(*kind, *at),
            NextTurn => {
                panic!("Cannot undo next turn");
            }
        }
    }
}
