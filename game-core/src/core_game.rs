use game_model::{
    game::Game,
    piece::{Piece, Power},
    Point2,
};

use crate::{
    command_handler::CommandHandler,
    game_controller::{GameCommand, GameController, MoveError},
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
        game_clone: Game,
        command_handler: &mut CommandHandler,
    ) -> CoreGameSubstate {
        let board = &game_clone.board;
        if !board.has_cell(target_point) {
            return CoreGameSubstate::Place;
        }

        match self {
            CoreGameSubstate::Place => {
                let place_command = GameCommand::PlacePiece(*target_point);
                match GameController::handle_command(game_clone.clone(), &place_command) {
                    Ok(_event) => {
                        command_handler.handle_new_event(game_clone, &place_command);
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
                                    let blast_command = GameCommand::Blast(*target_point);

                                    if let Ok(_game_action) = GameController::handle_command(
                                        game_clone.clone(),
                                        &blast_command,
                                    ) {
                                        command_handler
                                            .handle_new_event(game_clone, &blast_command);
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

                let move_command = GameCommand::MovePiece(*itself, *target_point);

                if let Ok(_) = GameController::handle_command(game_clone.clone(), &move_command) {
                    command_handler.handle_new_event(game_clone, &move_command);
                }
            }
            CoreGameSubstate::Activate(active_piece_pos) => {
                let shoot_command = GameCommand::TargetedShoot(*active_piece_pos, *target_point);
                if let Ok(_) = GameController::handle_command(game_clone.clone(), &shoot_command) {
                    command_handler.handle_new_event(game_clone, &shoot_command);
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

    use super::CoreGameSubstate;
    use crate::command_handler::CommandHandler;
    use game_events::{
        actions::compound_events::GameAction,
        event_broker::{EventBroker, EventConsumer},
    };

    #[test]
    fn test_place_single_piece() {
        let game = create_game_object();
        let mut event_broker = EventBroker::new();
        let event_log: Rc<RefCell<VecDeque<GameAction>>> = Rc::new(RefCell::new(VecDeque::new()));
        event_broker.subscribe(Box::new(EventLogger {
            events: event_log.clone(),
        }));

        let mut command_handler = CommandHandler::new(event_broker);

        let game_state = CoreGameSubstate::Place;
        game_state.on_click(&(0, 0).into(), game, &mut command_handler);

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
                id: 0,
                lost: false,
                unused_pieces: 2,
            },
            Team {
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
