use crate::{
    game_events::{CompoundEvent, EventConsumer, GameEventObject},
    rendering::animation::{
        Animation, PlacePieceAnimation,
    },
    BoardRender, CompoundEventType, GameEvent,
    GameEvent::{ChangeExhaustion, Place, Remove},
    PieceKind,
};
use macroquad::miniquad::info;
use std::{
    cell::{RefCell},
    rc::Rc,
};
use crate::game_events::AttackCompoundEvent;

pub struct RenderEventConsumer {
    pub(crate) board_render: Rc<RefCell<Box<BoardRender>>>,
}

impl RenderEventConsumer {
    pub(crate) fn handle_event_internal(&self, events: &[GameEvent], t: &CompoundEventType) {
        let mut board_render = (*self.board_render).borrow_mut();
        info!("Handling {} Render Events: {:?}", events.len(), t);
        match t {
            CompoundEventType::Attack(AttackCompoundEvent{ piece_kind, ..}) => {
                let mut animations: Vec<Animation> = vec![];
                let mut i = 0;
                let mut target_points = vec![];

                while let Some(GameEvent::Remove(to, _)) = events.get(i) {
                    target_points.push(*to);

                    i = i + 1;
                }

                let exhaustion_animation = if let Some(ChangeExhaustion(_from, to, at)) = events.get(i) {
                    i += 1;
                    for target in &target_points {
                        let mut bullet_animation = match piece_kind {
                            PieceKind::Queen => Animation::new_blast(*at),
                            PieceKind::HorizontalBar | PieceKind::VerticalBar | PieceKind::Sniper => {
                                Animation::new_bullet(*at, *target)
                            }
                            _ => panic!("Unknown piece_kind - can't generate bullet animation"),
                        };

                        bullet_animation
                            .next_animations
                            .push(Animation::new_remove(*target));

                        animations.push(bullet_animation);
                    }

                    Animation::new_exhaustion(*to, *at)
                } else {
                    panic!("CompoundEventType::Attack must start with an an Exhaust");
                };

                let merge_events = &events[i..];
                let mut merge_animations = Self::handle_merge_events(merge_events);

                for a in &mut animations {
                    // No idea how else to access a single elem of a vec mutably
                    a.next_animations.push(exhaustion_animation);
                    a.next_animations.append(&mut merge_animations);
                    break;
                }

                board_render.add_animation_sequence(animations);
            }
            CompoundEventType::Place(_) => {
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
            CompoundEventType::Move(_) => {
                let mut animations = vec![];
                let mut i = 0;

                if matches!(events.get(1), Some(Remove(_,_))) {
                    // Another piece was attacked during the move
                    if let Some(Remove(point, _piece)) = events.get(0) {
                        animations.push(Animation::new_remove(*point));
                        i += 1;
                    } else {
                        panic!(
                            "CompoundEventType::Move must begin with a Remove event but got: {:?}",
                            events.get(0)
                        )
                    };
                }

                let (_piece, from) = if let Some(Remove(point, piece)) = events.get(i) {
                    (*piece, *point)
                } else {
                    panic!(
                        "CompoundEventType::Move expected a Remove event but got: {:?}",
                        events.get(i)
                    )
                };

                let to = if let Some(Place(point, _piece)) = events.get(i+1) {
                    *point
                } else {
                    panic!(
                        "CompoundEventType::Move expected a Place event but got: {:?}",
                        events.get(i+1)
                    )
                };

                let mut move_animation = Animation::new_move(from, to);

                if let Some(GameEvent::ChangeExhaustion(_, to, point)) = events.get(i+2) {
                    move_animation.next_animations.push(Animation::new_exhaustion(*to,*point));
                } else {
                    panic!("CompoundEventType::Move expected an Exhaust event");
                }

                let merge_events = &events[i+3..];
                let mut merge_animations = Self::handle_merge_events(merge_events);

                move_animation.next_animations.append(&mut merge_animations);
                animations.push(move_animation);
                board_render.add_animation_sequence(animations);
            }
            CompoundEventType::Undo(_) => {
                let mut animations = vec![];
                for event in events {
                    match event {
                        GameEvent::AddUnusedPiece(team_id) => {
                            animations.push(Animation::new_add_unused(*team_id));
                        }
                        GameEvent::Place(point, piece) => {
                            animations.push(Animation::new_piece(
                                piece.team_id,
                                *point,
                                piece.piece_kind,
                                piece.exhaustion.is_done()
                            ));
                        }
                        GameEvent::Remove(point, _piece) => {
                            animations.push(Animation::new_remove(*point));
                        }
                        GameEvent::ChangeExhaustion(_, to, point) => {
                            animations.push(Animation::new_exhaustion(*to,*point));
                        }
                        GameEvent::AddEffect(kind, pos) => {
                            animations.push(Animation::new_add_effect(*kind, *pos))
                        }
                        GameEvent::RemoveEffect(kind, pos) => {
                            animations.push(Animation::new_remove_effect(*kind, *pos))
                        }
                        e => panic!("Unexpected subevent of CompoundEventType::Undo: {:?}", e),
                    };
                }

                board_render.add_animation_sequence(animations);
            }
            CompoundEventType::FinishTurn(_) => {
                let mut animations: Vec<Animation> = vec![];

                for event in events {
                    match event {
                        GameEvent::AddUnusedPiece(team_id) => {
                            let add_unused = Animation::new_add_unused(*team_id);

                            if let Some(c) = animations.first_mut() {
                                c.next_animations.push(add_unused);
                            } else {
                                animations.push(add_unused);
                            }
                        }
                        GameEvent::Place(point, piece) => {
                            animations.push(Animation::new_piece(
                                piece.team_id,
                                *point,
                                piece.piece_kind,
                                piece.exhaustion.is_done()
                            ));
                        }
                        GameEvent::NextTurn => {}
                        GameEvent::ChangeExhaustion(_,to,at) => {
                            animations.push(Animation::new_exhaustion(*to, *at));
                        }
                        e => panic!("Unexpected subevent of CompoundEventType::FinishTurn: {:?}", e),
                    };
                }

                board_render.add_animation_sequence(animations);
            }
        }
    }

    #[must_use]
    fn handle_merge_events(merge_events: &[GameEvent]) -> Vec<Animation> {
        let mut animations = vec![];
        let mut merged_points = vec![];
        for event in merge_events {
            match event {
                Remove(point, _piece) => {
                    merged_points.push(*point);
                }
                Place(point, piece) => {
                    for p in merged_points.iter() {
                        let mut a = Animation::new_move_towards(*p, *point);
                        a.next_animations.push(Animation::new_remove(*p));

                        animations.push(a);
                    }

                    let last_remove = animations.last_mut().unwrap().next_animations.last_mut().unwrap();
                    last_remove.next_animations
                        .push(
                        Animation::new_piece(
                            piece.team_id,
                            *point,
                            piece.piece_kind,
                            piece.exhaustion.is_done()
                        )
                    );
                }
                GameEvent::AddEffect(kind, pos) => {
                    let last_remove = animations.last_mut().unwrap().next_animations.last_mut().unwrap();

                    last_remove.next_animations
                        .push(
                            Animation::new_add_effect(*kind, *pos)
                        );
                }
                GameEvent::RemoveEffect(kind, pos) => {
                    animations.push(
                            Animation::new_remove_effect(*kind, *pos)
                        );
                }
                e => panic!("Unexpected subevent during merge phase: {:?}", e),
            };
        }

        info!("MERGE ANIMATIONS\n{:?}", animations);

        animations
    }
}

impl EventConsumer for RenderEventConsumer {
    fn handle_event(&mut self, event: &GameEventObject) {
        self.handle_event_internal(&event.event.get_compound_event().get_events(), &event.event);
    }
}
