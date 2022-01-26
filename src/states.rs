use crate::{
    game_events::{CompoundEventType, EventBroker},
    GameEvent::{CompoundEvent, Exhaust, Remove},
    *,
};


#[derive(Debug, Copy, Clone)]
pub enum State {
    Place,
    Move,
    Activate,
    Won(usize),
}

pub struct CoreGameState {
    pub selected: Option<Point2>,
    pub state: State,
}

impl CoreGameState {
    pub fn place() -> Self {
        CoreGameState {
            selected: None,
            state: State::Place,
        }
    }

    pub fn won(team: usize) -> Self {
        CoreGameState {
            selected: None,
            state: State::Won(team),
        }
    }

    pub fn move_piece(point: Point2) -> Self {
        CoreGameState {
            selected: Some(point),
            state: State::Move,
        }
    }

    pub fn activate(point: Point2) -> Self {
        CoreGameState {
            selected: Some(point),
            state: State::Activate,
        }
    }
}

impl CoreGameState {
    pub(crate) fn on_click(
        &self,
        target_point: &Point2,
        board: &mut Board,
        event_consumer: &mut EventBroker,
    ) -> CoreGameState {
        if !board.has_cell(target_point) {
            return Self::place();
        }

        match self.state {
            State::Place => {
                if let Some(target_piece) = board.get_piece_at(target_point) {
                    if target_piece.team_id == board.current_team_index {
                        if target_piece.can_move() {
                            return Self::move_piece(*target_point);
                        } else if target_piece.can_use_special() {
                            return Self::activate(*target_point);
                        }
                    }
                } else if board.unused_piece_available() {
                    let event = GameEvent::CompoundEvent(
                        vec![
                            GameEvent::RemoveUnusedPiece(board.current_team_index),
                            GameEvent::Place(
                                *target_point,
                                Piece::new(board.current_team_index, PieceKind::Simple),
                            ),
                        ],
                        CompoundEventType::Place,
                    );
                    event_consumer.handle_new_event(&event);
                }
            }
            State::Move => {
                if let Some(target_piece) = board.get_piece_at(target_point) {
                    if let Some(itself) = self.selected {
                        if itself == *target_point && target_piece.can_use_special() {
                            if let Some(activatable) = target_piece.activatable {
                                return match activatable.kind {
                                    Power::Blast => {
                                        let mut game_events = vec![Exhaust(true, *target_point)];
                                        for point in activatable.range.reachable_points(
                                            target_point,
                                            board,
                                            &RangeContext::Special(*target_piece),
                                        ) {
                                            if let Some(piece) = board.get_piece_at(&point) {
                                                game_events.push(Remove(point, *piece));
                                            }
                                        }

                                        event_consumer.handle_new_event(&CompoundEvent(
                                            game_events,
                                            CompoundEventType::Attack,
                                        ));
                                        Self::place()
                                    }
                                    Power::TargetedShoot => Self::activate(*target_point),
                                };
                            }
                        }
                    }
                    if target_piece.team_id == board.current_team_index && target_piece.can_move() {
                        return Self::move_piece(*target_point);
                    }
                }

                let selected_point = self.selected.unwrap();

                let selected_piece = board.get_piece_at(&selected_point).unwrap();
                if let Some(m) = selected_piece.movement.as_ref() {
                    if m.range
                        .reachable_points(
                            &selected_point,
                            board,
                            &RangeContext::Moving(*selected_piece),
                        )
                        .contains(target_point)
                    {
                        event_consumer.handle_new_event(&GameEvent::new_move(
                            *selected_piece,
                            self.selected.unwrap(),
                            *target_point,
                        ));
                    }
                }
            }
            State::Activate => {
                let active_piece_pos = self.selected.as_ref().unwrap();
                let active_piece = board.get_piece_at(active_piece_pos).unwrap();
                if let Some(target_piece) = board.get_piece_at(target_point) {
                    if target_piece.team_id != board.current_team_index
                        && active_piece.can_use_special()
                    {
                        event_consumer.handle_new_event(&CompoundEvent(
                            vec![
                                Exhaust(true, *active_piece_pos),
                                Remove(*target_point, *target_piece),
                            ],
                            CompoundEventType::Attack,
                        ));
                    }
                }
            }
            State::Won(team) => {
                return Self::won(team);
            }
        }

        Self::place()
    }

    pub(crate) fn merge_patterns(board: &mut Board, event_consumer: &mut EventBroker) {
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
                                let matched_piece = board.get_piece_mut_at(point).unwrap();
                                event_consumer
                                    .handle_new_event(&GameEvent::Remove(*point, *matched_piece));
                                matched_piece.dying = true;
                            });

                            let new_piece = Piece::new(any_team_id, pattern.turn_into);

                            let new_piece_x = x as u8 + pattern.new_piece_relative_position.x;
                            let new_piece_y = y as u8 + pattern.new_piece_relative_position.y;

                            event_consumer.handle_new_event(&GameEvent::Place(
                                Point2::new(new_piece_x, new_piece_y),
                                new_piece,
                            ));

                            println!(
                                "Matched pattern at {}:{}; new piece at {}:{}",
                                x, y, new_piece_x, new_piece_y
                            );
                        }
                    }
                }
            }
        }
    }
}
