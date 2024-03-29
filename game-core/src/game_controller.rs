use nanoserde::{DeJson, SerJson};
use std::{
    collections::HashSet,
    fmt::{Display, Formatter},
};

use game_model::{
    board::Board,
    game::Game,
    piece::{Piece, PieceKind},
    Point2,
};

use crate::board_event_consumer::BoardEventConsumer;
use game_events::actions::{
    attack::AttackBuilder,
    compound_events::{CompoundEventBuilder, FlushResult, GameAction},
    merge::MergeBuilder,
    moving::MoveBuilder,
    place::EffectBuilder,
};
use game_model::pattern::Pattern;

pub(crate) struct GameController {}

#[derive(Debug, Copy, Clone)]
pub enum MoveError {
    PieceAlreadyPresent(Piece),
    NoPiecePresent,
    NotSupportedByPiece,
    NoPieceAvailable,
    IllegalMove,
}

#[derive(Debug, Copy, Clone, SerJson, DeJson)]
pub enum GameCommand {
    InitPlayer(u8),
    PlacePiece(Point2),
    MovePiece(Point2, Point2),
    Blast(Point2),
    TargetedShoot(Point2, Point2),
    NextTurn,
    Undo,
}

impl Display for GameCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GameCommand::InitPlayer(start_pieces) => write!(f, "Init({})", start_pieces),
            GameCommand::PlacePiece(at) => write!(f, "Place{}", at),
            GameCommand::MovePiece(from, to) => write!(f, "Move{}{}", from, to),
            GameCommand::Blast(at) => write!(f, "Blast{}", at),
            GameCommand::TargetedShoot(from, to) => write!(f, "Shoot{}{}", from, to),
            GameCommand::NextTurn => write!(f, "NextTurn"),
            GameCommand::Undo => write!(f, "Undo"),
        }
    }
}

type MoveResult = Result<GameAction, MoveError>;

impl GameController {
    pub fn handle_command(mut game: Game, command: &GameCommand) -> MoveResult {
        let game = &mut game;
        match command {
            GameCommand::InitPlayer(add_unused) => Self::init_player(game, *add_unused),
            GameCommand::PlacePiece(pos) => Self::place_piece(game, pos),
            GameCommand::MovePiece(from, target_point) => {
                Self::move_piece(game, from, target_point)
            }
            GameCommand::Blast(piece_pos) => Self::blast(game, piece_pos),
            GameCommand::TargetedShoot(attacking_piece_pos, target_pos) => {
                Self::targeted_shoot(game, attacking_piece_pos, target_pos)
            }
            GameCommand::NextTurn => Ok(Self::next_turn(game)),
            GameCommand::Undo => todo!(),
        }
    }

    pub fn init_player(game: &mut Game, add_unused: u8) -> MoveResult {
        if game.current_team().unused_pieces != 0
            || !game.board.placed_pieces(game.current_team_index).is_empty()
        {
            // TODO: Find better way to assert that the game hasn't started yet
            return Err(MoveError::IllegalMove);
        }

        let mut builder = GameAction::finish_turn();
        for _ in 0..add_unused {
            builder.add_unused_piece(game.current_team_index);
        }

        Ok(builder.build())
    }

    pub fn place_piece(game: &mut Game, pos: &Point2) -> MoveResult {
        if let Some(target_piece) = game.board.get_piece_at(pos) {
            return MoveResult::Err(MoveError::PieceAlreadyPresent(*target_piece));
        }

        if !game.unused_piece_available() {
            return MoveResult::Err(MoveError::NoPieceAvailable);
        }

        let new_piece = Piece::new(game.current_team_index, PieceKind::Simple);
        let mut place_event = GameAction::place(*pos, new_piece, game.current_team_index);

        push_effects_if_present(&mut place_event, &game.board, &new_piece, &pos);

        MoveResult::Ok(flush_and_merge(game, Box::new(place_event)))
    }

    pub fn move_piece(game: &mut Game, from: &Point2, target_point: &Point2) -> MoveResult {
        let selected_piece = game
            .board
            .get_piece_at(from)
            .ok_or(MoveError::NoPiecePresent)?;

        let m = selected_piece
            .movement
            .as_ref()
            .ok_or(MoveError::NotSupportedByPiece)?;

        if !selected_piece.can_move() || *from == *target_point {
            return MoveResult::Err(MoveError::IllegalMove);
        }

        if !m
            .range
            .reachable_points(&from, &game.board)
            .contains(target_point)
        {
            return MoveResult::Err(MoveError::NotSupportedByPiece);
        }

        let mut move_event = MoveBuilder::new(*from, *target_point, *selected_piece);

        if let Some(target_piece) = game.board.get_piece_at(target_point) {
            move_event.remove_piece(*target_piece);

            if target_piece.team_id == game.current_team_index {
                // is actually atm already checked by reachable_points but not sure if this should be relied on
                return MoveResult::Err(MoveError::IllegalMove);
            }

            remove_effects_if_present(&mut move_event, &game.board, target_piece, target_point);
        }

        return MoveResult::Ok(flush_and_merge(game, Box::new(move_event)));
    }

    pub fn blast(game: &mut Game, piece_pos: &Point2) -> MoveResult {
        let attacking_piece = game
            .board
            .get_piece_at(piece_pos)
            .ok_or(MoveError::NoPiecePresent)?;

        let activatable = attacking_piece
            .activatable
            .ok_or(MoveError::NotSupportedByPiece)?;

        let mut attack_event = AttackBuilder::new(attacking_piece, *piece_pos);
        let reachable_points = activatable.range.reachable_points(piece_pos, &game.board);

        if reachable_points.is_empty() {
            return MoveResult::Err(MoveError::IllegalMove);
        }

        for point in reachable_points {
            if let Some(piece) = game.board.get_piece_at(&point) {
                attack_event.remove_piece(point, *piece);
                remove_effects_if_present(&mut attack_event, &game.board, piece, &point);
            }
        }

        MoveResult::Ok(flush_and_merge(game, Box::new(attack_event)))
    }

    pub fn targeted_shoot(
        game: &mut Game,
        attacking_piece_pos: &Point2,
        target_pos: &Point2,
    ) -> MoveResult {
        let active_piece = game
            .board
            .get_piece_at(attacking_piece_pos)
            .ok_or(MoveError::NoPiecePresent)?;

        let target_piece = game
            .board
            .get_piece_at(target_pos)
            .ok_or(MoveError::NoPiecePresent)?;

        if let Some(activatable) = active_piece.activatable {
            if !activatable
                .range
                .reachable_points_for_piece(attacking_piece_pos, active_piece, &game.board)
                .contains(target_pos)
            {
                return MoveResult::Err(MoveError::IllegalMove);
            }
        } else {
            return MoveResult::Err(MoveError::NotSupportedByPiece);
        }

        let mut exhaustion_clone = target_piece.exhaustion.clone();
        exhaustion_clone.on_attack();

        let mut attack_event = AttackBuilder::new(active_piece, *attacking_piece_pos);
        attack_event.remove_piece(*target_pos, *target_piece);

        remove_effects_if_present(&mut attack_event, &game.board, target_piece, target_pos);

        return Ok(flush_and_merge(game, Box::new(attack_event)));
    }

    pub fn next_turn(game: &Game) -> GameAction {
        let mut finish_turn = GameAction::finish_turn();
        {
            let current_team_index = game.current_team_index;

            finish_turn
                .add_unused_piece(current_team_index)
                .add_unused_piece(current_team_index);

            game.board.for_each_placed_piece(|point, piece| {
                if piece.movement.is_none() && piece.activatable.is_none() {
                    return;
                }

                let mut exhaustion_clone = piece.exhaustion.clone();
                exhaustion_clone.reset();

                if exhaustion_clone != piece.exhaustion {
                    finish_turn.change_exhaustion(piece.exhaustion, exhaustion_clone, point);
                }
            });
        }
        finish_turn.build()
    }
}

fn push_effects_if_present(
    effect_builder: &mut dyn EffectBuilder,
    board: &Board,
    new_piece: &Piece,
    pos: &Point2,
) {
    if let Some(effect) = new_piece.effect {
        effect
            .range
            .reachable_points_for_piece(pos, new_piece, &board)
            .iter()
            .for_each(|&point| {
                effect_builder.add_effect(point);
            });
    }
}

fn remove_effects_if_present(
    effect_builder: &mut dyn EffectBuilder,
    board: &Board,
    piece: &Piece,
    pos: &Point2,
) {
    if let Some(effect) = piece.effect {
        effect
            .range
            .reachable_points(pos, &board)
            .iter()
            .for_each(|&point| {
                effect_builder.remove_effect(point);
            });
    }
}

fn merge_patterns(board: &Board, merge_builder: &mut MergeBuilder) {
    let mut dying: HashSet<Point2> = HashSet::new();
    for pattern in &Pattern::all_patterns() {
        for x in 0..board.w as usize - pattern.components[0].len() + 1 {
            for y in 0..board.h as usize - pattern.components.len() + 1 {
                let matched = pattern.match_board(&board, x as u8, y as u8);

                if let Some(matched_entities) = matched {
                    let any_team_id = board.get_piece_at(&matched_entities[0]).unwrap().team_id;
                    if matched_entities
                        .iter()
                        .map(|point| board.get_piece_at(point).unwrap())
                        .all(|piece| piece.team_id == any_team_id)
                        && !matched_entities.iter().any(|p| dying.contains(p))
                    {
                        let new_piece = Piece::new(any_team_id, pattern.turn_into);

                        let new_piece_x = x as u8 + pattern.new_piece_relative_position.x;
                        let new_piece_y = y as u8 + pattern.new_piece_relative_position.y;

                        let new_piece_pos = Point2::new(new_piece_x, new_piece_y);
                        merge_builder.place_piece(new_piece_pos, new_piece);

                        matched_entities.iter().for_each(|point| {
                            // println!("Going to remove matched piece {:?}", matched_piece);
                            {
                                let matched_piece = board.get_piece_at(point).unwrap();
                                merge_builder.remove_piece(*point, *matched_piece);
                                dying.insert(*point);
                            }
                            let matched_piece = board.get_piece_at(point).unwrap();

                            remove_effects_if_present(merge_builder, board, matched_piece, point);
                        });

                        push_effects_if_present(merge_builder, &board, &new_piece, &new_piece_pos);

                        /* println!(
                            "Matched pattern at {}:{}; new piece at {}:{}",
                            x, y, new_piece_x, new_piece_y
                        );*/

                        return; // ensures merges are handled sequentially. In future parallel merges may be nice
                    }
                }
            }
        }
    }
}

fn flush_and_merge(game: &mut Game, event_builder: Box<dyn CompoundEventBuilder>) -> GameAction {
    let mut flush_result = BoardEventConsumer::flush(game, event_builder);
    while let FlushResult::Merge(mut m) = flush_result {
        merge_patterns(&mut game.board, &mut m);
        flush_result = BoardEventConsumer::flush(game, Box::new(m));
    }
    if let FlushResult::Build(game_action) = flush_result {
        // always true
        game_action
    } else {
        panic!("unreachable code")
    }
}
