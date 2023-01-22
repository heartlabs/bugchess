use std::fmt::Display;

use crate::{
    actions::{
        compound_events::{CompoundEvent, CompoundEventBuilder, GameAction},
        merge::{MergeBuilder, MergeCompoundEvent},
        place::EffectBuilder,
    },
    atomic_events::AtomicEvent,
};
use derive_getters::Getters;
use game_model::{
    piece::{EffectKind, Exhaustion, Piece},
    Point2,
};
use nanoserde::{DeJson, SerJson};

use super::compound_events::FlushResult;

#[derive(Debug, Clone, SerJson, DeJson, Getters)]
pub struct MoveCompoundEvent {
    from: Point2,
    to: Point2,
    moved_piece: Piece,
    exhaustion_afterwards: Exhaustion,
    captured_piece: Option<Piece>,
    added_effects: Vec<Point2>,
    removed_effects: Vec<Point2>,

    merge_events: Option<MergeCompoundEvent>,
}

pub struct MoveBuilder {
    event: MoveCompoundEvent,
}

impl MoveBuilder {
    pub fn new(from: Point2, to: Point2, moved_piece: Piece) -> Self {
        let mut exhaustion_afterwards = moved_piece.exhaustion.clone();
        exhaustion_afterwards.on_move();

        MoveBuilder {
            event: MoveCompoundEvent {
                from,
                to,
                moved_piece,
                exhaustion_afterwards,
                captured_piece: None,
                added_effects: vec![],
                removed_effects: vec![],

                merge_events: None,
            },
        }
    }

    pub fn remove_piece(&mut self, piece: Piece) -> &mut Self {
        self.event.captured_piece = Some(piece);

        self
    }
}

impl CompoundEvent for MoveCompoundEvent {
    fn get_events(&self) -> Vec<AtomicEvent> {
        let mut all_events: Vec<AtomicEvent> = vec![];

        if let Some(captured_piece) = self.captured_piece {
            all_events.push(AtomicEvent::Remove(self.to, captured_piece));
        }

        all_events.push(AtomicEvent::Remove(self.from, self.moved_piece));
        all_events.push(AtomicEvent::Place(self.to, self.moved_piece));

        all_events.push(AtomicEvent::ChangeExhaustion(
            self.moved_piece.exhaustion,
            self.exhaustion_afterwards,
            self.to,
        ));

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

impl CompoundEventBuilder for MoveBuilder {
    fn build_with_merge_event(mut self: Box<Self>, merge_event: MergeCompoundEvent) -> GameAction {
        self.event.merge_events = Some(merge_event);
        self.build()
    }

    fn build(self) -> GameAction {
        GameAction::Move(self.event)
    }

    fn flush(self: Box<Self>, consumer: &mut dyn FnMut(&AtomicEvent)) -> FlushResult {
        self.event.get_events().iter().for_each(consumer);

        FlushResult::Merge(MergeBuilder::new(self))
    }
}

impl EffectBuilder for MoveBuilder {
    fn add_effect(&mut self, at: Point2) {
        self.event.added_effects.push(at);
    }

    fn remove_effect(&mut self, at: Point2) {
        self.event.removed_effects.push(at);
    }
}

impl Display for MoveCompoundEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let action = match self.captured_piece {
            Some(_) => "captures",
            None => "moves to",
        };
        write!(f, "{} {} {}", self.from(), action, self.to())?;

        if let Some(merge) = &self.merge_events {
            if !merge.placed_pieces().is_empty() {
                write!(f, " with {}", merge)?;
            }
        }

        Ok(())
    }
}
