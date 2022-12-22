use std::fmt::Display;

use crate::{
    actions::{
        compound_events::{CompoundEvent, CompoundEventBuilder, GameAction},
        merge::{MergeBuilder, MergeCompoundEvent},
    },
    atomic_events::AtomicEvent,
};
use derive_getters::Getters;
use game_model::{
    board::Point2,
    piece::{EffectKind, Piece},
};
use nanoserde::{DeBin, SerBin};

use super::compound_events::FlushResult;

#[derive(Debug, Clone, SerBin, DeBin, Getters)]
pub struct PlaceCompoundEvent {
    at: Point2,
    piece: Piece,
    team_id: usize,
    added_effects: Vec<Point2>,

    merge_events: Option<MergeCompoundEvent>,
}

pub struct PlaceBuilder {
    event: PlaceCompoundEvent,
}

pub trait EffectBuilder {
    fn add_effect(&mut self, at: Point2);
    fn remove_effect(&mut self, at: Point2);
}

impl CompoundEventBuilder for PlaceBuilder {
    fn build_with_merge_event(mut self: Box<Self>, merge_event: MergeCompoundEvent) -> GameAction {
        self.event.merge_events = Some(merge_event);
        self.build()
    }

    fn build(self) -> GameAction {
        GameAction::Place(self.event)
    }

    fn flush(self: Box<Self>, consumer: &mut dyn FnMut(&AtomicEvent)) -> FlushResult {
        self.event.get_events().iter().for_each(consumer);

        FlushResult::Merge(MergeBuilder::new(self))
    }
}

impl PlaceBuilder {
    pub(crate) fn new(at: Point2, piece: Piece, team_id: usize) -> Self {
        PlaceBuilder {
            event: PlaceCompoundEvent {
                merge_events: None,
                at,
                piece,
                added_effects: vec![],
                team_id,
            },
        }
    }
}

impl EffectBuilder for PlaceBuilder {
    fn add_effect(&mut self, at: Point2) {
        self.event.added_effects.push(at);
    }

    fn remove_effect(&mut self, _at: Point2) {
        panic!("No effect can be removed during 'Place'")
    }
}

impl CompoundEvent for PlaceCompoundEvent {
    fn get_events(&self) -> Vec<AtomicEvent> {
        let mut all_events: Vec<AtomicEvent> = vec![];

        all_events.push(AtomicEvent::Place(self.at, self.piece));
        all_events.push(AtomicEvent::RemoveUnusedPiece(self.team_id));

        for effect in self.added_effects.iter() {
            all_events.push(AtomicEvent::AddEffect(EffectKind::Protection, *effect));
        }

        if let Some(merge_events) = &self.merge_events {
            all_events.extend(&merge_events.get_events());
        }
        all_events
    }
}

impl Display for PlaceCompoundEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Placing {} at {}", self.piece(), self.at())?;

        if let Some(merge) = &self.merge_events {
            if !merge.placed_pieces().is_empty() {
                write!(f, " with {}", merge)?;
            }
        }

        Ok(())
    }
}
