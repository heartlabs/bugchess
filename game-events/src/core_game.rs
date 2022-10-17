// TODO: That doesnt belong in the events crate - maybe put it in its own crate?
use game_model::{board::Point2, game::Game, piece::Power};

use crate::{
    actions::compound_events::GameAction,
    game_controller::{GameController, MoveError}, event_broker::EventBroker,
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
        game: &mut Game, // TODO: Operate on a clone of Game instead and re-perform events on actual game outside?
        event_broker: &mut EventBroker,
        //event_option: &mut Option<GameAction>,
    ) -> CoreGameSubstate {
        let board = &game.board;
        if !board.has_cell(target_point) {

            return CoreGameSubstate::Place;
        }

        match self {
            CoreGameSubstate::Place => {
                match GameController::place_piece(game, target_point) {
                    Ok(event) => {
                        //std::mem::drop(game);
                        event_broker.handle_new_event(&event);
                    }
                    Err(MoveError::PieceAlreadyPresent(target_piece)) => {
                        if target_piece.team_id == game.current_team_index {
                            if target_piece.can_move() {
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
                                        GameController::blast(game, target_point)
                                    {
                                        std::mem::drop(game);
                                        event_broker.handle_new_event(&game_action);
                                    }

                                    CoreGameSubstate::Place
                                }
                                Power::TargetedShoot => CoreGameSubstate::Activate(*target_point),
                            };
                        }
                    }
                    if target_piece.team_id == game.current_team_index && target_piece.can_move() {
                        return CoreGameSubstate::Move(*target_point);
                    }
                }

                if let Ok(game_action) = GameController::move_piece(game, itself, target_point) {
                    std::mem::drop(game);
                    event_broker.handle_new_event(&game_action);
                }
            }
            CoreGameSubstate::Activate(active_piece_pos) => {
                if let Ok(game_action) =
                    GameController::targeted_shoot(game, active_piece_pos, target_point)
                {
                    std::mem::drop(game);
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


#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, cell::RefCell, rc::Rc};

    use game_model::{game::{Game, Team}, piece::PieceKind};

    use crate::{event_broker::{EventBroker}, game_events::{GameEventObject, EventConsumer}, actions::compound_events::GameAction, board_event_consumer::BoardEventConsumer};
    use super::CoreGameSubstate;

    #[test]
    fn test_place_single_piece() {
        let mut game = create_game_object();
        let sender_id = "1".to_string();
        let mut event_broker = EventBroker::new(sender_id);
        let event_log: Rc<RefCell<VecDeque<GameEventObject>>> = Rc::new(RefCell::new(VecDeque::new()));
        event_broker.subscribe(Box::new(EventLogger { events: event_log.clone() }));

        let game_state = CoreGameSubstate::Place;
        game_state.on_click(&(0,0).into(), &mut game, &mut event_broker);

        let log: &VecDeque<GameEventObject> = &(*event_log).borrow();
        assert!(log.len() == 1, "Logged events: {:?}", log);

        if let GameAction::Place(p) = &log[0].event {
            assert!(p.piece().piece_kind == PieceKind::Simple);
            assert!(p.at() == &(0,0).into());
            assert!(p.team_id() == &0);
        } else {
            panic!("Expected place event but was {:?}", log[0].event)
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
        pub events: Rc<RefCell<VecDeque<GameEventObject>>>
    }

    impl EventConsumer for EventLogger {
        fn handle_event(&mut self, event: &GameEventObject) {
            (*self.events).borrow_mut().push_back(event.clone());
        }
    }
}