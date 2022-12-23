// TODO: That doesnt belong in the events crate - maybe put it in its own crate?
use game_model::{
    board::Point2,
    game::Game,
    piece::{Piece, Power},
};

use crate::{
    event_broker::EventBroker,
    game_controller::{GameController, MoveError},
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CoreGameSubstate {
    Place,
    Move(Point2),
    Activate(Point2),
    Won(usize),
    Wait,
}

impl CoreGameSubstate {
    pub fn on_click(
        &self,
        target_point: &Point2,
        mut game_clone: Game,
        event_broker: &mut EventBroker,
    ) -> CoreGameSubstate {
        let board = &game_clone.board;
        if !board.has_cell(target_point) {
            return CoreGameSubstate::Place;
        }

        //        std::mem::drop(game_ref);

        match self {
            CoreGameSubstate::Place => {
                match GameController::place_piece(&mut game_clone, target_point) {
                    Ok(event) => {
                        //std::mem::drop(game);
                        event_broker.handle_new_event(&event);
                    }
                    Err(MoveError::PieceAlreadyPresent(target_piece)) => {
                        if target_piece.team_id == game_clone.current_team_index {
                            if target_piece.can_move() || can_blast(&target_piece) {
                                return CoreGameSubstate::Move(*target_point);
                            } else if target_piece.can_use_special() {
                                return CoreGameSubstate::Activate(*target_point);
                            }
                        }
                    }
                    Err(MoveError::NoPieceAvailable) => {
                        return CoreGameSubstate::Place;
                    }
                    Err(error) => {
                        panic!("Unexpected error {:?}", error)
                    }
                };
            }
            CoreGameSubstate::Move(itself) => {
                if let Some(target_piece) = board.get_piece_at(target_point) {
                    if *itself == *target_point && target_piece.can_use_special() {
                        if let Some(activatable) = target_piece.activatable {
                            return match activatable.kind {
                                Power::Blast => {
                                    if let Ok(game_action) =
                                        GameController::blast(&mut game_clone, target_point)
                                    {
                                        std::mem::drop(game_clone);
                                        event_broker.handle_new_event(&game_action);
                                    }

                                    CoreGameSubstate::Place
                                }
                                Power::TargetedShoot => CoreGameSubstate::Activate(*target_point),
                            };
                        }
                    }
                    if target_piece.team_id == game_clone.current_team_index
                        && target_piece.can_move()
                    {
                        return CoreGameSubstate::Move(*target_point);
                    }
                }

                if let Ok(game_action) =
                    GameController::move_piece(&mut game_clone, itself, target_point)
                {
                    std::mem::drop(game_clone);
                    event_broker.handle_new_event(&game_action);
                }
            }
            CoreGameSubstate::Activate(active_piece_pos) => {
                if let Ok(game_action) =
                    GameController::targeted_shoot(&mut game_clone, active_piece_pos, target_point)
                {
                    std::mem::drop(game_clone);
                    event_broker.handle_new_event(&game_action);
                }
            }
            CoreGameSubstate::Won(team) => {
                return CoreGameSubstate::Won(*team);
            }
            CoreGameSubstate::Wait => return CoreGameSubstate::Wait,
        }

        CoreGameSubstate::Place
    }
}

fn can_blast(piece: &Piece) -> bool {
    if let Some(activatable) = piece.activatable {
        return piece.can_use_special() && activatable.kind == Power::Blast;
    }

    false
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::VecDeque, rc::Rc};

    use game_model::{
        game::{Game, Team},
        piece::PieceKind,
    };

    use super::{CoreGameSubstate, EventBroker};
    use game_events::{actions::compound_events::GameAction, game_events::EventConsumer};

    #[test]
    fn test_place_single_piece() {
        let game = create_game_object();
        let mut event_broker = EventBroker::new();
        let event_log: Rc<RefCell<VecDeque<GameAction>>> = Rc::new(RefCell::new(VecDeque::new()));
        event_broker.subscribe(Box::new(EventLogger {
            events: event_log.clone(),
        }));

        let game_state = CoreGameSubstate::Place;
        game_state.on_click(&(0, 0).into(), game, &mut event_broker);

        let log: &VecDeque<GameAction> = &(*event_log).borrow();
        assert!(log.len() == 1, "Logged events: {:?}", log);

        if let GameAction::Place(p) = &log[0] {
            assert!(p.piece().piece_kind == PieceKind::Simple);
            assert!(p.at() == &(0, 0).into());
            assert!(p.team_id() == &0);
        } else {
            panic!("Expected place event but was {:?}", log[0])
        }
    }

    pub fn create_game_object() -> Game {
        let teams = vec![
            Team {
                name: "Red",
                id: 0,
                lost: false,
                unused_pieces: 2,
            },
            Team {
                name: "Yellow",
                id: 1,
                lost: false,
                unused_pieces: 2,
            },
        ];
        Game::new(teams, 8, 8)
    }

    pub struct EventLogger {
        pub events: Rc<RefCell<VecDeque<GameAction>>>,
    }

    impl EventConsumer for EventLogger {
        fn handle_event(&mut self, event: &GameAction) {
            (*self.events).borrow_mut().push_back(event.clone());
        }
    }
}
