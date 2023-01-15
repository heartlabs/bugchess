use game_events::{
    actions::{compound_events::GameAction, merge::MergeCompoundEvent},
    atomic_events::AtomicEvent,
    game_events::*,
};
use game_model::{
    board::Point2,
    piece::{EffectKind, PieceKind},
};

use crate::{
    animation::{Animation, PlacePieceAnimation},
    BoardRender,
};
use std::{cell::RefCell, rc::Rc};

pub struct RenderEventConsumer {
    pub(crate) board_render: Rc<RefCell<BoardRender>>,
}

impl RenderEventConsumer {
    pub fn new(board_render: &Rc<RefCell<BoardRender>>) -> Self {
        RenderEventConsumer {
            board_render: board_render.clone(),
        }
    }

    pub(crate) fn handle_event_internal(&self, t: &GameAction) {
        let mut board_render = (*self.board_render).borrow_mut();
        match t {
            GameAction::Attack(attack_event) => {
                let mut animations: Vec<Animation> = vec![];

                for (target, _) in attack_event.removed_pieces() {
                    animations.push(attack_animation(
                        attack_event.piece_kind(),
                        *attack_event.attacking_piece_pos(),
                        *target,
                    ));
                }

                let exhaustion_animation = Animation::new_exhaustion(
                    *attack_event.exhaustion_afterwards(),
                    *attack_event.attacking_piece_pos(),
                );

                let mut merge_animations = attack_event
                    .merge_events()
                    .as_ref()
                    .map(Self::handle_merge_events)
                    .unwrap_or(vec![]);

                let first_animation = &mut animations[0];
                first_animation.next_animations.push(exhaustion_animation);
                first_animation
                    .next_animations
                    .append(&mut merge_animations);

                board_render.add_animation_sequence(animations);
            }
            GameAction::Place(place_event) => {
                let place_piece =
                    PlacePieceAnimation::new(*place_event.team_id(), *place_event.at());

                let mut animation = Animation::new(Box::new(place_piece));

                if let Some(merge_events) = place_event.merge_events() {
                    animation
                        .next_animations
                        .append(&mut Self::handle_merge_events(merge_events));
                }

                board_render.add_animation_sequence(vec![animation]);
            }
            GameAction::Move(move_event) => {
                let mut animations = vec![];

                if let Some(_) = move_event.captured_piece() {
                    let mut remove_animation = Animation::new_die(*move_event.to());

                    for pos in move_event.removed_effects() {
                        remove_animation
                            .next_animations
                            .push(Animation::new_remove_effect(EffectKind::Protection, *pos));
                    }

                    animations.push(remove_animation);
                }

                let mut move_animation = Animation::new_move(*move_event.from(), *move_event.to());

                for pos in move_event.added_effects() {
                    move_animation
                        .next_animations
                        .push(Animation::new_add_effect(EffectKind::Protection, *pos));
                }

                move_animation
                    .next_animations
                    .push(Animation::new_exhaustion(
                        *move_event.exhaustion_afterwards(),
                        *move_event.to(),
                    ));

                if let Some(merge_events) = move_event.merge_events() {
                    move_animation
                        .next_animations
                        .append(&mut Self::handle_merge_events(merge_events));
                }
                animations.push(move_animation);
                board_render.add_animation_sequence(animations);
            }
            GameAction::Undo(_) => {
                let mut animations = vec![];
                for event in &t.get_compound_event().get_events() {
                    match event {
                        AtomicEvent::AddUnusedPiece(team_id) => {
                            animations.push(Animation::new_add_unused(*team_id));
                        }
                        AtomicEvent::Place(point, piece) => {
                            animations.push(Animation::new_piece(
                                piece.team_id,
                                *point,
                                piece.piece_kind,
                                piece.exhaustion.is_done(),
                            ));
                        }
                        AtomicEvent::Remove(point, _piece) => {
                            animations.push(Animation::new_remove(*point));
                        }
                        AtomicEvent::ChangeExhaustion(_, to, point) => {
                            animations.push(Animation::new_exhaustion(*to, *point));
                        }
                        AtomicEvent::AddEffect(kind, pos) => {
                            animations.push(Animation::new_add_effect(*kind, *pos))
                        }
                        AtomicEvent::RemoveEffect(kind, pos) => {
                            animations.push(Animation::new_remove_effect(*kind, *pos))
                        }
                        e => panic!("Unexpected subevent of CompoundEventType::Undo: {:?}", e),
                    };
                }

                board_render.add_animation_sequence(animations);
            }
            GameAction::FinishTurn(_) => {
                let mut animations: Vec<Animation> = vec![];

                for event in &t.get_compound_event().get_events() {
                    match event {
                        AtomicEvent::AddUnusedPiece(team_id) => {
                            let add_unused = Animation::new_add_unused(*team_id);

                            if let Some(c) = animations.first_mut() {
                                c.next_animations.push(add_unused);
                            } else {
                                animations.push(add_unused);
                            }
                        }
                        AtomicEvent::Place(point, piece) => {
                            animations.push(Animation::new_piece(
                                piece.team_id,
                                *point,
                                piece.piece_kind,
                                piece.exhaustion.is_done(),
                            ));
                        }
                        AtomicEvent::NextTurn => {}
                        AtomicEvent::ChangeExhaustion(_, to, at) => {
                            animations.push(Animation::new_exhaustion(*to, *at));
                        }
                        e => panic!(
                            "Unexpected subevent of CompoundEventType::FinishTurn: {:?}",
                            e
                        ),
                    };
                }

                board_render.add_animation_sequence(animations);
            }
        }
    }

    #[must_use]
    fn handle_merge_events(merge_events: &MergeCompoundEvent) -> Vec<Animation> {
        let mut animations = vec![];

        for (merged_towards, piece) in merge_events.placed_pieces() {
            for (merged_from, _) in merge_events.removed_pieces().iter() {
                let mut a = Animation::new_move_towards(*merged_from, *merged_towards);
                a.next_animations.push(Animation::new_remove(*merged_from));

                animations.push(a);
            }

            let last_remove = animations
                .last_mut()
                .unwrap()
                .next_animations
                .last_mut()
                .unwrap();
            last_remove.next_animations.push(Animation::new_piece(
                piece.team_id,
                *merged_towards,
                piece.piece_kind,
                piece.exhaustion.is_done() && piece.piece_kind != PieceKind::Castle,
            ));
        }

        for pos in merge_events.added_effects() {
            let last_remove = animations
                .last_mut()
                .unwrap()
                .next_animations
                .last_mut()
                .unwrap();

            last_remove
                .next_animations
                .push(Animation::new_add_effect(EffectKind::Protection, *pos));
        }

        for pos in merge_events.removed_effects() {
            animations.push(Animation::new_remove_effect(EffectKind::Protection, *pos));
        }

        if let Some(merge_event) = merge_events.merge_events() {
            animations[0]
                .next_animations
                .extend(Self::handle_merge_events(merge_event));
        }

        animations
    }
}

fn attack_animation(piece_kind: &PieceKind, pos: Point2, target: Point2) -> Animation {
    let mut bullet_animation = match piece_kind {
        PieceKind::Queen => Animation::new_blast(pos),
        PieceKind::HorizontalBar | PieceKind::VerticalBar | PieceKind::Sniper => {
            Animation::new_bullet(pos, target)
        }
        _ => panic!("Unknown piece_kind - can't generate bullet animation"),
    };
    bullet_animation
        .next_animations
        .push(Animation::new_die(target));
    bullet_animation
}

impl EventConsumer for RenderEventConsumer {
    fn handle_event(&mut self, event: &GameAction) {
        self.handle_event_internal(&event);
    }
}
