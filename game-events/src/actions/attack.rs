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
    piece::{EffectKind, Exhaustion, Piece, PieceKind},
    Point2,
};
use nanoserde::{DeJson, SerJson};

use super::compound_events::FlushResult;

#[derive(Debug, Clone, SerJson, DeJson, Getters)]
pub struct AttackCompoundEvent {
    piece_kind: PieceKind,
    attacking_piece_pos: Point2,
    exhaustion_before: Exhaustion,
    exhaustion_afterwards: Exhaustion,
    removed_pieces: Vec<(Point2, Piece)>,
    added_effects: Vec<Point2>,
    removed_effects: Vec<Point2>,

    merge_events: Option<MergeCompoundEvent>,
}

pub struct AttackBuilder {
    event: AttackCompoundEvent,
}

impl AttackBuilder {
    pub fn new(piece: &Piece, piece_pos: Point2) -> Self {
        let mut exhaustion_afterwards = piece.exhaustion;
        exhaustion_afterwards.on_attack();
        AttackBuilder {
            event: AttackCompoundEvent {
                piece_kind: piece.piece_kind,
                attacking_piece_pos: piece_pos,
                exhaustion_before: piece.exhaustion,
                exhaustion_afterwards,
                removed_pieces: vec![],
                removed_effects: vec![],
                added_effects: vec![],

                merge_events: None,
            },
        }
    }

    pub fn remove_piece(&mut self, point: Point2, piece: Piece) -> &mut Self {
        self.event.removed_pieces.push((point, piece));
        self
    }
}

impl CompoundEvent for AttackCompoundEvent {
    fn get_events(&self) -> Vec<AtomicEvent> {
        let mut all_events: Vec<AtomicEvent> = vec![];

        for (at, piece) in self.removed_pieces.iter() {
            all_events.push(AtomicEvent::Remove(*at, *piece));
        }

        all_events.push(AtomicEvent::ChangeExhaustion(
            self.exhaustion_before,
            self.exhaustion_afterwards,
            self.attacking_piece_pos,
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

impl CompoundEventBuilder for AttackBuilder {
    fn build_with_merge_event(mut self: Box<Self>, merge_event: MergeCompoundEvent) -> GameAction {
        self.event.merge_events = Some(merge_event);
        self.build()
    }

    fn build(self) -> GameAction {
        if self.event.removed_pieces.is_empty() {
            panic!(
                "Can't build an AttackCompoundEvent without any removed pieces: {:?}",
                self.event
            );
        }

        GameAction::Attack(self.event)
    }

    fn flush(self: Box<Self>, consumer: &mut dyn FnMut(&AtomicEvent)) -> FlushResult {
        self.event.get_events().iter().for_each(consumer);

        FlushResult::Merge(MergeBuilder::new(self))
    }
}

impl EffectBuilder for AttackBuilder {
    fn add_effect(&mut self, at: Point2) {
        self.event.added_effects.push(at);
    }

    fn remove_effect(&mut self, at: Point2) {
        self.event.removed_effects.push(at);
    }
}

impl Display for AttackCompoundEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let targets: Vec<&Point2> = self.removed_pieces().iter().map(|(p, _)| p).collect();
        write!(f, "{} Attacks {:?}", self.attacking_piece_pos(), targets)?;

        if let Some(merge) = &self.merge_events
            && !merge.placed_pieces().is_empty() {
                write!(f, " with {}", merge)?;
            }

        Ok(())
    }
}
