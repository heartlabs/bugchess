use std::fmt::Display;

use crate::{
    actions::{
        compound_events::{CompoundEvent, CompoundEventBuilder, GameAction},
        place::EffectBuilder,
    },
    atomic_events::AtomicEvent,
};
use derive_getters::Getters;
use game_model::{
    piece::{EffectKind, Piece},
    Point2,
};
use nanoserde::{DeJson, SerJson};

use super::compound_events::FlushResult;

// TODO: there should be a current and n past events with separate merges
pub struct MergeBuilder {
    event: MergeCompoundEvent,
    super_event: Box<dyn CompoundEventBuilder>,
}

impl MergeBuilder {
    pub(crate) fn new(super_event: Box<dyn CompoundEventBuilder>) -> Self {
        MergeBuilder {
            event: MergeCompoundEvent::new(),
            super_event,
        }
    }

    pub fn remove_piece(&mut self, point: Point2, piece: Piece) {
        self.event.removed_pieces.push((point, piece));
    }

    pub fn place_piece(&mut self, point: Point2, piece: Piece) {
        self.event.placed_pieces.push((point, piece));
    }
}

#[derive(Debug, Clone, SerJson, DeJson, Getters)]
pub struct MergeCompoundEvent {
    placed_pieces: Vec<(Point2, Piece)>,
    removed_pieces: Vec<(Point2, Piece)>,
    added_effects: Vec<Point2>,
    removed_effects: Vec<Point2>,

    merge_events: Option<Box<MergeCompoundEvent>>,
}

impl MergeCompoundEvent {
    pub fn new() -> Self {
        MergeCompoundEvent {
            merge_events: None,
            placed_pieces: vec![],
            removed_pieces: vec![],
            added_effects: vec![],
            removed_effects: vec![],
        }
    }
}

impl CompoundEvent for MergeCompoundEvent {
    fn get_events(&self) -> Vec<AtomicEvent> {
        let mut all_events: Vec<AtomicEvent> = vec![];

        for (at, piece) in self.removed_pieces.iter() {
            all_events.push(AtomicEvent::Remove(*at, *piece));
        }
        for (at, piece) in self.placed_pieces.iter() {
            all_events.push(AtomicEvent::Place(*at, *piece));
        }

        for effect in self.removed_effects.iter() {
            all_events.push(AtomicEvent::RemoveEffect(EffectKind::Protection, *effect));
        }
        for effect in self.added_effects.iter() {
            all_events.push(AtomicEvent::AddEffect(EffectKind::Protection, *effect));
        }

        if let Some(merge_events) = &self.merge_events {
            all_events.extend(&merge_events.get_events());
        }
        all_events
    }
}

impl EffectBuilder for MergeBuilder {
    fn add_effect(&mut self, at: Point2) {
        self.event.added_effects.push(at);
    }

    fn remove_effect(&mut self, at: Point2) {
        self.event.removed_effects.push(at);
    }
}

impl CompoundEventBuilder for MergeBuilder {
    fn build_with_merge_event(mut self: Box<Self>, merge_event: MergeCompoundEvent) -> GameAction {
        self.event.merge_events = Some(Box::new(merge_event));
        self.build()
    }

    fn build(self) -> GameAction {
        self.super_event.build_with_merge_event(self.event)
    }

    fn flush(self: Box<Self>, consumer: &mut dyn FnMut(&AtomicEvent)) -> FlushResult {
        let events = self.event.get_events();
        if events.is_empty() {
            return FlushResult::Build(self.build());
        }

        events.iter().for_each(consumer);

        FlushResult::Merge(MergeBuilder::new(self))
    }
}

impl Display for MergeCompoundEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Merging to ")?;

        self.placed_pieces.iter().for_each(|(point, piece)| {
            _ = write!(f, "{} at {}; ", piece, point);
        });

        Ok(())
    }
}
