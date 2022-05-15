use std::rc::Rc;
use std::cell::{RefCell, RefMut};
use macroquad::miniquad::info;
use crate::{BoardRender, CompoundEventType, GameEvent};
use crate::game_events::{CompoundEvent, EventConsumer, GameEventObject};
use crate::GameEvent::{Place, Remove};
use crate::rendering::animation::{Animation, AnimationExpert, MovePieceAnimation, NewPieceAnimation, PlacePieceAnimation, RemovePieceAnimation};
use crate::rendering::PieceRender;

pub struct RenderEventConsumer {
    pub(crate) board_render: Rc<RefCell<Box<BoardRender>>>,
}

impl RenderEventConsumer {
    pub(crate) fn handle_event_internal(&self, events: &Vec<GameEvent>, t: &CompoundEventType) {
        let mut board_render = (*self.board_render).borrow_mut();
        info!("Handling {} Render Events: {:?}", events.len(), t);
        match t {
            CompoundEventType::Attack => {
                for event in events {
                    match event {
                        GameEvent::Exhaust (_,_) => {
                        }
                        GameEvent::Remove(point,piece) => {
                            board_render.add_animation(Animation::new(Box::new(RemovePieceAnimation { at: *point })));
                        }
                        e => panic!("Unexpected subevent of CompoundEventType::Merge: {:?}", e)
                    };
                }
            }
            CompoundEventType::Place => {
                let place_piece = if let Some(Place(point, piece)) = events.get(0) {
                    PlacePieceAnimation::new(piece.team_id, *point)
                } else {
                    panic!("CompoundEventType::Place must begin with a Place event but got: {:?}", events.get(0))
                };
                let mut animation = Animation::new(Box::new(place_piece));

                board_render.add_animation(animation);
                if let Some(GameEvent::RemoveUnusedPiece(_)) = events.get(1) {
                    //
                } else {
                    panic!("CompoundEventType::Place must have an RemoveUnusedPiece as second event");
                }


                let merge_events = &events[2..];
                Self::handle_merge_events(&mut board_render, merge_events);
            }
            CompoundEventType::Move => {
                let (piece, from) = if let Some(Remove(point, piece)) = events.get(0) {
                    (*piece, *point)
                } else {
                    panic!("CompoundEventType::Move must begin with a Remove event but got: {:?}", events.get(0))
                };

                let to = if let Some(Place(point, piece)) = events.get(1) {
                    *point
                } else {
                    panic!("CompoundEventType::Move must have a Place event at second but got: {:?}", events.get(1))
                };

                let mut animation = Animation::new(Box::new(MovePieceAnimation {from, to}));

                board_render.add_animation(animation);
                if let Some(GameEvent::Exhaust(_,_)) = events.get(2) {
                    //
                } else {
                    panic!("CompoundEventType::Place must have an Exhaust as third event");
                }


                let merge_events = &events[3..];
                Self::handle_merge_events(&mut board_render, merge_events);
            }
            CompoundEventType::Undo(t) => {
                for event in events {
                    match event {
                        GameEvent::AddUnusedPiece (team_id) => {
                            board_render.add_unused_piece(*team_id);
                        }
                        GameEvent::Place(point,piece) => {
                            board_render.add_placed_piece(point, piece.piece_kind, piece.team_id);
                        }
                        GameEvent::Remove(point,piece) => {
                            board_render.placed_pieces.remove(point);
                        }
                        GameEvent::AddUnusedPiece(team) => {
                            board_render.add_unused_piece(*team);
                        }
                        GameEvent::UndoExhaustion(p, point) => {}
                        e => panic!("Unexpected subevent of CompoundEventType::Undo: {:?}", e)
                    };
                }
            }
            CompoundEventType::FinishTurn => {
                for event in events {
                    match event {
                        GameEvent::AddUnusedPiece (team_id) => {
                            board_render.add_unused_piece(*team_id);
                        }
                        GameEvent::Place(point,piece) => {
                            board_render.add_placed_piece(point, piece.piece_kind, piece.team_id);
                        }
                        GameEvent::NextTurn => {}
                        e => panic!("Unexpected subevent of CompoundEventType::Merge: {:?}", e)
                    };
                }
            }
        }
    }

    fn handle_merge_events(board_render: &mut RefMut<Box<BoardRender>>, merge_events: &[GameEvent]) {
        for event in merge_events {
            match event {
                GameEvent::Remove(point, piece) => {
                    board_render.add_animation(Animation::new(Box::new(
                        RemovePieceAnimation { at: *point }
                    )));
                }
                GameEvent::Place(point, piece) => {
                    board_render.add_animation(Animation::new(Box::new(
                        NewPieceAnimation { piece_kind: piece.piece_kind, to: *point, team: piece.team_id }
                    )));
                }
                e => panic!("Unexpected subevent during merge phase: {:?}", e)
            };
        }
    }
}

impl EventConsumer for RenderEventConsumer {
    fn handle_event(&mut self, event: &GameEventObject) {
        let CompoundEvent { events: events, kind: t } = &event.event;
        self.handle_event_internal(events, t);
    }
}
