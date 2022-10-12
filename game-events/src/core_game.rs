use std::{collections::HashSet, default};

// TODO: That doesnt belong in the events crate - maybe put it in its own crate?
use game_model::{
    board::{Board, Pattern, Point2},
    game::Game,
    piece::{Piece, PieceKind, Power},
    ranges::RangeContext,
};

use crate::{
    actions::{
        attack::AttackBuilder,
        compound_events::{CompoundEventBuilder, GameAction, FlushResult},
        merge::MergeBuilder,
        moving::MoveBuilder,
        place::EffectBuilder,
    }, 
    board_event_consumer::BoardEventConsumer, 
    game_controller::{GameController, MoveError},
};

#[derive(Debug, Copy, Clone)]
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
        event_option: &mut Option<GameAction>,
    ) -> CoreGameSubstate {
        let board = &game.board;
        if !board.has_cell(target_point) {
            return CoreGameSubstate::Place;
        }

        match self {
            CoreGameSubstate::Place => {
                match GameController::place_piece(game, target_point) {
                    Ok(event) => {
                        let _ = event_option.insert(event);
                    },
                    Err(MoveError::PieceAlreadyPresent(target_piece)) => {
                        if target_piece.team_id == game.current_team_index {
                            if target_piece.can_move() {
                                return CoreGameSubstate::Move(*target_point);
                            } else if target_piece.can_use_special() {
                                return CoreGameSubstate::Activate(*target_point);
                            } 
                        }
                    },
                    Err(error) => {panic!("Unexpected error {:?}", error)},
                };
            }
            CoreGameSubstate::Move(itself) => {
                if let Some(target_piece) = board.get_piece_at(target_point) {
                    if *itself == *target_point && target_piece.can_use_special() {
                        if let Some(activatable) = target_piece.activatable {
                            return match activatable.kind {
                                Power::Blast => {
                                    if let Ok(game_action) = GameController::blast(game, target_point) {
                                        let _ = event_option.insert(game_action);
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
                    let _ = event_option.insert(game_action);
                }
            }
            CoreGameSubstate::Activate(active_piece_pos) => {
                if let Ok(game_action) = GameController::targeted_shoot(game, active_piece_pos, target_point) {
                    let _ = event_option.insert(game_action);
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