use crate::{
    game_events::{CompoundEvent, EventConsumer, GameEventObject},
    rendering::animation::{
        Animation, AnimationExpert, MovePieceAnimation, NewPieceAnimation, PlacePieceAnimation,
        RemovePieceAnimation,
    },
    BoardRender, CompoundEventType, GameEvent,
    GameEvent::{Exhaust, Place, Remove},
    PieceKind, Power,
};
use macroquad::miniquad::info;
use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

pub struct RenderEventConsumer {
    pub(crate) board_render: Rc<RefCell<Box<BoardRender>>>,
}

impl RenderEventConsumer {
    pub(crate) fn handle_event_internal(&self, events: &Vec<GameEvent>, t: &CompoundEventType) {
        let mut board_render = (*self.board_render).borrow_mut();
        info!("Handling {} Render Events: {:?}", events.len(), t);
        match t {
            CompoundEventType::Attack(piece_kind) => {
                let from = if let Some(Exhaust(_, from)) = events.get(0) {
                    from
                } else {
                    panic!("CompoundEventType::Attack must start with an an Exhaust");
                };

                let mut i = 1;
                let mut animations: Vec<Animation> = vec![];

                while let Some(GameEvent::Remove(to, _)) = events.get(i) {
                    let mut bullet_animation = match piece_kind {
                        PieceKind::Queen => Animation::new_blast(*from),
                        PieceKind::HorizontalBar | PieceKind::VerticalBar | PieceKind::Sniper => {
                            Animation::new_bullet(*from, *to)
                        }
                        _ => panic!("Unknown piece_kind - can't generate bullet animation"),
                    };

                    bullet_animation
                        .next_animations
                        .push(Animation::new_remove(*to));

                    animations.push(bullet_animation);

                    i = i + 1;
                }

                let merge_events = &events[i..];
                let mut merge_animations = Self::handle_merge_events(merge_events);

                for mut a in &mut animations {
                    // No idea how else to access a single elem of a vec mutably
                    a.next_animations.append(&mut merge_animations);
                    break;
                }

                board_render.add_animation_sequence(animations);
            }
            CompoundEventType::Place => {
                let place_piece = if let Some(Place(point, piece)) = events.get(0) {
                    PlacePieceAnimation::new(piece.team_id, *point)
                } else {
                    panic!(
                        "CompoundEventType::Place must begin with a Place event but got: {:?}",
                        events.get(0)
                    )
                };
                let mut animation = Animation::new(Box::new(place_piece));

                if let Some(GameEvent::RemoveUnusedPiece(_)) = events.get(1) {
                    //
                } else {
                    panic!(
                        "CompoundEventType::Place must have an RemoveUnusedPiece as second event"
                    );
                }

                let merge_events = &events[2..];
                let mut merge_animations = Self::handle_merge_events(merge_events);

                animation.next_animations.append(&mut merge_animations);

                board_render.add_animation_sequence(vec![animation]);
            }
            CompoundEventType::Move => {
                let (piece, from) = if let Some(Remove(point, piece)) = events.get(0) {
                    (*piece, *point)
                } else {
                    panic!(
                        "CompoundEventType::Move must begin with a Remove event but got: {:?}",
                        events.get(0)
                    )
                };

                let to = if let Some(Place(point, piece)) = events.get(1) {
                    *point
                } else {
                    panic!(
                        "CompoundEventType::Move must have a Place event at second but got: {:?}",
                        events.get(1)
                    )
                };

                if let Some(GameEvent::Exhaust(_, _)) = events.get(2) {
                    //
                } else {
                    panic!("CompoundEventType::Place must have an Exhaust as third event");
                }

                let merge_events = &events[3..];
                let mut merge_animations = Self::handle_merge_events(merge_events);

                let mut move_animation = Animation::new_move(from, to);
                move_animation.next_animations.append(&mut merge_animations);
                board_render.add_animation_sequence(vec![move_animation]);
            }
            CompoundEventType::Undo(t) => {
                let mut animations = vec![];
                for event in events {
                    match event {
                        GameEvent::AddUnusedPiece(team_id) => {
                            board_render.add_unused_piece(*team_id);
                        }
                        GameEvent::Place(point, piece) => {
                            animations.push(Animation::new_piece(
                                piece.team_id,
                                *point,
                                piece.piece_kind,
                            ));
                        }
                        GameEvent::Remove(point, piece) => {
                            animations.push(Animation::new_remove(*point));
                        }
                        GameEvent::UndoExhaustion(p, point) => {}
                        e => panic!("Unexpected subevent of CompoundEventType::Undo: {:?}", e),
                    };
                }

                board_render.add_animation_sequence(animations);
            }
            CompoundEventType::FinishTurn => {
                let mut animations = vec![];

                for event in events {
                    match event {
                        GameEvent::AddUnusedPiece(team_id) => {
                            board_render.add_unused_piece(*team_id);
                        }
                        GameEvent::Place(point, piece) => {
                            animations.push(Animation::new_piece(
                                piece.team_id,
                                *point,
                                piece.piece_kind,
                            ));
                        }
                        GameEvent::NextTurn => {}
                        e => panic!("Unexpected subevent of CompoundEventType::Merge: {:?}", e),
                    };
                }

                board_render.add_animation_sequence(animations);
            }
        }
    }

    #[must_use]
    fn handle_merge_events(merge_events: &[GameEvent]) -> Vec<Animation> {
        let mut animations = vec![];
        for event in merge_events {
            match event {
                Remove(point, piece) => {
                    animations.push(Animation::new_remove(*point));
                }
                Place(point, piece) => {
                    animations.push(Animation::new_piece(
                        piece.team_id,
                        *point,
                        piece.piece_kind,
                    ));
                }
                e => panic!("Unexpected subevent during merge phase: {:?}", e),
            };
        }

        animations
    }
}

impl EventConsumer for RenderEventConsumer {
    fn handle_event(&mut self, event: &GameEventObject) {
        let CompoundEvent { events, kind: t } = &event.event;
        self.handle_event_internal(events, t);
    }
}
