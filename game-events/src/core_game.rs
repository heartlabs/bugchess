// TODO: That doesnt belong in the events crate - maybe put it in its own crate?
use game_model::{
    board::{Board, Pattern, Point2},
    game::Game,
    piece::{EffectKind, Piece, PieceKind, Power},
    ranges::RangeContext,
};

use crate::{
    actions::{
        attack::AttackBuilder,
        compound_events::{CompoundEventBuilder, GameAction},
        merge::MergeBuilder,
        moving::MoveBuilder,
        place::EffectBuilder,
    },
    atomic_events::AtomicEvent,
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
        game: &mut Game,
        event_option: &mut Option<Box<dyn CompoundEventBuilder>>,
    ) -> CoreGameSubstate {
        let board = &game.board;
        if !board.has_cell(target_point) {
            return CoreGameSubstate::Place;
        }

        match self {
            CoreGameSubstate::Place => {
                if let Some(target_piece) = board.get_piece_at(target_point) {
                    if target_piece.team_id == game.current_team_index {
                        if target_piece.can_move() {
                            return CoreGameSubstate::Move(*target_point);
                        } else if target_piece.can_use_special() {
                            return CoreGameSubstate::Activate(*target_point);
                        }
                    }
                } else if game.unused_piece_available() {
                    let new_piece = Piece::new(game.current_team_index, PieceKind::Simple);
                    let mut place_event =
                        GameAction::place(*target_point, new_piece, game.current_team_index);

                    Self::push_effects_if_present(
                        &mut place_event,
                        &board,
                        &new_piece,
                        target_point,
                    );

                    let _ = event_option.insert(Box::new(place_event));
                }
            }
            CoreGameSubstate::Move(itself) => {
                if let Some(target_piece) = board.get_piece_at(target_point) {
                    if *itself == *target_point && target_piece.can_use_special() {
                        if let Some(activatable) = target_piece.activatable {
                            return match activatable.kind {
                                Power::Blast => {
                                    let mut attack_event =
                                        AttackBuilder::new(target_piece, *target_point);
                                    for point in activatable.range.reachable_points(
                                        target_point,
                                        board,
                                        &RangeContext::Special(*target_piece),
                                    ) {
                                        if let Some(piece) = board.get_piece_at(&point) {
                                            attack_event.remove_piece(point, *piece);
                                            Self::remove_effects_if_present(
                                                &mut attack_event,
                                                board,
                                                piece,
                                                &point,
                                            );
                                        }
                                    }

                                    let _ = event_option.insert(Box::new(attack_event));

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

                let selected_piece = board.get_piece_at(&itself).unwrap();
                if let Some(m) = selected_piece.movement.as_ref() {
                    if m.range
                        .reachable_points(&itself, board, &RangeContext::Moving(*selected_piece))
                        .contains(target_point)
                    {
                        let mut move_event =
                            MoveBuilder::new(*itself, *target_point, *selected_piece);

                        if let Some(target_piece) = board.get_piece_at(target_point) {
                            move_event.remove_piece(*target_piece);
                        }

                        if let Some(target_piece) = board.get_piece_at(target_point) {
                            Self::remove_effects_if_present(
                                &mut move_event,
                                board,
                                target_piece,
                                target_point,
                            );
                        }

                        let _ = event_option.insert(Box::new(move_event));
                    }
                }
            }
            CoreGameSubstate::Activate(active_piece_pos) => {
                let active_piece = board.get_piece_at(active_piece_pos).unwrap();
                if let Some(target_piece) = board.get_piece_at(target_point) {
                    if target_piece.team_id != game.current_team_index
                        && active_piece.can_use_special()
                    {
                        let mut exhaustion_clone = target_piece.exhaustion.clone();
                        exhaustion_clone.on_attack();

                        let mut attack_event = AttackBuilder::new(active_piece, *active_piece_pos);
                        attack_event.remove_piece(*target_point, *target_piece);

                        Self::remove_effects_if_present(
                            &mut attack_event,
                            board,
                            target_piece,
                            target_point,
                        );

                        let _ = event_option.insert(Box::new(attack_event));
                    }
                }
            }
            CoreGameSubstate::Won(team) => {
                return CoreGameSubstate::Won(*team);
            }
            CoreGameSubstate::Wait => return CoreGameSubstate::Wait,
        }

        CoreGameSubstate::Place
    }

    fn push_effects_if_present(
        event_composer: &mut dyn EffectBuilder,
        board: &Board,
        new_piece: &Piece,
        pos: &Point2,
    ) {
        if let Some(effect) = new_piece.effect {
            effect
                .range
                .reachable_points(pos, &board, &RangeContext::Area)
                .iter()
                .for_each(|&point| {
                    event_composer.add_effect(point);
                });
        }
    }

    fn remove_effects_if_present(
        event_composer: &mut dyn EffectBuilder,
        board: &Board,
        piece: &Piece,
        pos: &Point2,
    ) {
        if let Some(effect) = piece.effect {
            effect
                .range
                .reachable_points(pos, &board, &RangeContext::Area)
                .iter()
                .for_each(|&point| {
                    event_composer.remove_effect(point);
                });
        }
    }

    // TODO: Can we make this private?
    pub fn merge_patterns(board: &mut Board, event_composer: &mut MergeBuilder) {
        //let mut remove_pieces = vec![];
        //let mut place_pieces = vec![];

        for pattern in &Pattern::all_patterns() {
            for x in 0..board.w as usize - pattern.components[0].len() + 1 {
                for y in 0..board.h as usize - pattern.components.len() + 1 {
                    let matched = { board.match_pattern(&pattern, x as u8, y as u8) };

                    if let Some(mut matched_entities) = matched {
                        let any_team_id = board.get_piece_at(&matched_entities[0]).unwrap().team_id;
                        println!("Pattern matched!");
                        if matched_entities
                            .iter()
                            .map(|point| board.get_piece_at(point).unwrap())
                            .all(|piece| piece.team_id == any_team_id && !piece.dying)
                        {
                            matched_entities.iter_mut().for_each(|point| {
                                // println!("Going to remove matched piece {:?}", matched_piece);
                                {
                                    let matched_piece = board.get_piece_mut_at(point).unwrap();
                                    event_composer.remove_piece(*point, *matched_piece);
                                    matched_piece.dying = true;
                                }
                                let matched_piece = board.get_piece_at(point).unwrap();

                                Self::remove_effects_if_present(
                                    event_composer,
                                    board,
                                    matched_piece,
                                    point,
                                );
                            });

                            let new_piece = Piece::new(any_team_id, pattern.turn_into);

                            let new_piece_x = x as u8 + pattern.new_piece_relative_position.x;
                            let new_piece_y = y as u8 + pattern.new_piece_relative_position.y;

                            let new_piece_pos = Point2::new(new_piece_x, new_piece_y);
                            event_composer.place_piece(new_piece_pos, new_piece);

                            Self::push_effects_if_present(
                                event_composer,
                                &board,
                                &new_piece,
                                &new_piece_pos,
                            );

                            /* println!(
                                "Matched pattern at {}:{}; new piece at {}:{}",
                                x, y, new_piece_x, new_piece_y
                            );*/
                        }
                    }
                }
            }
        }
    }
}
