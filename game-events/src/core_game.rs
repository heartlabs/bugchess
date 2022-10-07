// TODO: That doesnt belong in the events crate - maybe put it in its own crate?
use game_logic::{
    board::{Point2, Board, Pattern}, 
    game::Game, 
    piece::{Piece, PieceKind, Power, EffectKind}, 
    ranges::RangeContext
};

use crate::{actions::compound_events::GameAction, atomic_events::AtomicEvent};


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
        event_option: &mut Option<GameAction>,
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
                    let mut place_event = GameAction::place();
                    place_event
                        .get_compound_event_mut()
                        .push_event(AtomicEvent::Place(*target_point, new_piece));

                    place_event
                        .get_compound_event_mut()
                        .push_event(AtomicEvent::RemoveUnusedPiece(game.current_team_index));

                    Self::push_effects_if_present(
                        &mut place_event,
                        &board,
                        &new_piece,
                        target_point,
                    );

                    let _ = event_option.insert(place_event);
                }
            }
            CoreGameSubstate::Move(itself) => {
                if let Some(target_piece) = board.get_piece_at(target_point) {
                    if *itself == *target_point && target_piece.can_use_special() {
                        if let Some(activatable) = target_piece.activatable {
                            return match activatable.kind {
                                Power::Blast => {
                                    let mut exhaustion_clone = target_piece.exhaustion.clone();
                                    exhaustion_clone.on_attack();
                                    let mut attack_event =
                                        GameAction::attack(target_piece.piece_kind);
                                    for point in activatable.range.reachable_points(
                                        target_point,
                                        board,
                                        &RangeContext::Special(*target_piece),
                                    ) {
                                        if let Some(piece) = board.get_piece_at(&point) {
                                            attack_event
                                                .get_compound_event_mut()
                                                .push_event(AtomicEvent::Remove(point, *piece));
                                            Self::remove_effects_if_present(
                                                &mut attack_event,
                                                board,
                                                piece,
                                                &point,
                                            );
                                        }
                                    }

                                    attack_event.get_compound_event_mut().push_event(
                                        AtomicEvent::ChangeExhaustion(
                                            target_piece.exhaustion,
                                            exhaustion_clone,
                                            *target_point,
                                        ),
                                    );

                                    let _ = event_option.insert(attack_event);

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
                        let mut move_event = GameAction::moving();

                        if let Some(target_piece) = board.get_piece_at(target_point) {
                            move_event
                                .get_compound_event_mut()
                                .push_event(AtomicEvent::Remove(*target_point, *target_piece));
                        }

                        move_event
                            .get_compound_event_mut()
                            .push_event(AtomicEvent::Remove(*itself, *selected_piece));
                        move_event
                            .get_compound_event_mut()
                            .push_event(AtomicEvent::Place(*target_point, *selected_piece));

                        if let Some(target_piece) = board.get_piece_at(target_point) {
                            Self::remove_effects_if_present(
                                &mut move_event,
                                board,
                                target_piece,
                                target_point,
                            );
                        }

                        let mut exhaustion_clone = selected_piece.exhaustion.clone();
                        exhaustion_clone.on_attack();

                        move_event.get_compound_event_mut().push_event(
                            AtomicEvent::ChangeExhaustion(
                                selected_piece.exhaustion,
                                exhaustion_clone,
                                *target_point,
                            ),
                        );

                        let _ = event_option.insert(move_event);
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

                        let mut attack_event = GameAction::attack(active_piece.piece_kind);
                        attack_event
                            .get_compound_event_mut()
                            .push_event(AtomicEvent::Remove(*target_point, *target_piece));
                        attack_event.get_compound_event_mut().push_event(
                            AtomicEvent::ChangeExhaustion(
                                target_piece.exhaustion,
                                exhaustion_clone,
                                *active_piece_pos,
                            ),
                        );

                        Self::remove_effects_if_present(
                            &mut attack_event,
                            board,
                            target_piece,
                            target_point,
                        );

                        let _ = event_option.insert(attack_event);
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
        event_composer: &mut GameAction,
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
                    event_composer
                        .get_compound_event_mut()
                        .push_event(AtomicEvent::AddEffect(EffectKind::Protection, point))
                });
        }
    }

    fn remove_effects_if_present(
        event_composer: &mut GameAction,
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
                    event_composer
                        .get_compound_event_mut()
                        .push_event(AtomicEvent::RemoveEffect(EffectKind::Protection, point))
                });
        }
    }

    // TODO: Can we make this private?
    pub fn merge_patterns(board: &mut Board, event_composer: &mut GameAction) {
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
                                    event_composer
                                        .get_compound_event_mut()
                                        .push_event(AtomicEvent::Remove(*point, *matched_piece));
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
                            event_composer
                                .get_compound_event_mut()
                                .push_event(AtomicEvent::Place(new_piece_pos, new_piece));

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