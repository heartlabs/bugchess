use std::cell::RefCell;
use std::rc::Rc;
use crate::game_events::{EventConsumer, GameEvent};
use crate::rendering::CustomRenderContext;
use crate::{constants::*, Direction, piece::*, Range, RangeContext};
use macroquad::prelude::*;
use nanoserde::{SerBin, DeBin};

#[derive(Clone, Copy)]
pub struct Team {
    pub id: usize,
    pub color: Color,
    pub name: &'static str,
    pub lost: bool,
    pub unused_pieces: u8,
}

#[derive(Clone)]
pub struct Cell {
    pub point: Point2,
    pub piece: Option<Piece>,
    pub effects: Vec<EffectKind>,
}

pub struct Board {
    //pub(crate) placed_pieces: Vec<Vec<Option<Piece>>>,
    pub(crate) cells: Vec<Vec<Cell>>,
    pub(crate) teams: Vec<Team>,
    pub(crate) current_team_index: usize,
    pub w: u8,
    pub h: u8,
}

impl Board {

    pub fn new(teams: Vec<Team>) -> Board {
        let mut cells = vec![];

        for x in 0..BOARD_WIDTH {
            let mut column = vec![];
            for y in 0..BOARD_HEIGHT {
                column.push(Cell { point: Point2::new(x,y), piece: Option::None, effects: vec![] });
            }
            cells.push(column);
        }

        Board {
            cells,
            teams,
            current_team_index: 0,
            w: BOARD_WIDTH,
            h: BOARD_HEIGHT,
        }
    }


    pub fn for_each_cell_mut<F>(&mut self, mut closure: F)  where F : FnMut(&mut Cell) -> () {
        for mut row in self.cells.iter_mut() {
            for cell in row {
                closure(cell);
            }
        }
    }

    pub fn for_each_placed_piece_mut<F>(&mut self, mut closure: F)  where F : FnMut(Point2, &mut Piece) -> () {
        self.for_each_cell_mut(|cell| {
            if let Some(piece) = cell.piece.as_mut() {
                closure(cell.point, piece);
            }
        });
    }
    pub fn for_each_cell<F>(&self, mut closure: F)  where F : FnMut(&Cell) -> () {
        for mut row in self.cells.iter() {
            for cell in row {
                closure(cell);
            }
        }
    }

    pub fn for_each_placed_piece<F>(&self, mut closure: F)  where F : FnMut(Point2, &Piece) -> () {
        self.for_each_cell(|cell| {
                if let Some(piece) = cell.piece.as_ref() {
                    closure(cell.point, piece);
                }
        });
    }

    pub fn num_unused_pieces(&self) -> u8 {
        self.current_team().unused_pieces
    }
    pub fn num_unused_pieces_of(&self, team_id: usize) -> u8 {
        self.get_team(team_id).unused_pieces
    }


    pub fn has_effect_at(&self, effect: &EffectKind, pos: &Point2) -> bool {
        self.get_cell(pos).effects.contains(effect)
    }

    pub fn get_piece_at(&self, pos: &Point2) -> Option<&Piece> {
        if !self.has_cell(pos) {
            return Option::None;
        }
        self.cells[pos.x as usize][pos.y as usize].piece.as_ref()
    }

    pub fn get_piece_mut(&mut self, x: u8, y: u8) -> Option<&mut Piece> {
        self.cells[x as usize][y as usize].piece.as_mut()
    }

    pub fn get_piece_mut_at(&mut self, pos: &Point2) -> Option<&mut Piece> {
        self.get_piece_mut(pos.x, pos.y)
    }

    pub fn has_cell(&self, point: &Point2) -> bool {
        point.x < self.w && point.y < self.h
    }

    pub fn current_team(&self) -> Team {
        self.teams[self.current_team_index]
    }

    pub fn is_current_team(&self, team_id: usize) -> bool {
        self.current_team_index == team_id
    }

    pub fn get_team(&self, team_id: usize) -> &Team {
        &self.teams[team_id]
    }

    pub fn num_teams(&self) -> usize {
        self.teams.len()
    }

    pub fn mark_team_as_lost(&mut self, team_id: usize) {
        println!("Team {} lost.", team_id);
        self.teams[team_id].lost = true;
    }

    pub fn next_team(&mut self) -> Option<Team> {
        let initial_team_index = self.current_team_index;
        println!("From team {} to next team.", self.current_team_index);

        loop {
            self.current_team_index += 1;
            if self.current_team_index >= self.teams.len() {
                self.current_team_index = 0;
            }

            if self.current_team_index == initial_team_index {
                println!("No next team. Current team {}", self.current_team_index);
                return None; // All (other) teams lost
            } else if !self.current_team().lost {
                println!("Next team is {}", self.current_team_index);
                return Some(self.current_team());
            }
        }
    }

    pub fn match_pattern(
        &self,
        pattern: &Pattern,
        start_x: u8,
        start_y: u8,
    ) -> Option<Vec<Point2>> {
        let mut matched_entities = Vec::new();
        for (pattern_y, line) in pattern.components.iter().enumerate() {
            for (pattern_x, p) in line.iter().enumerate() {
                let board_x = start_x + pattern_x as u8;
                let board_y = start_y + pattern_y as u8;

                let board_point = Point2::new(board_x, board_y);

                if let Some(_piece) = self.get_piece_at(&board_point) {
                    if p == &PatternComponent::Free {
                        return None;
                    } else if p == &PatternComponent::OwnPiece {
                        matched_entities.push(board_point);
                    }
                } else if p == &PatternComponent::OwnPiece {
                    return None;
                }
            }
        }

        Option::Some(matched_entities)
    }

    // publicly accessible with events:

    pub(crate) fn add_unused_piece_for(&mut self, team_id: usize) {
        self.teams[team_id].unused_pieces += 1;
    }

    pub fn remove_unused_piece(&mut self, team_id: usize) -> bool {
        if self.teams[team_id].unused_pieces <= 0 {
            false
        } else {
            self.teams[team_id].unused_pieces -= 1;
            true
        }
    }

    pub fn unused_piece_available(&self) -> bool {
        self.current_team().unused_pieces > 0
    }

    pub(crate) fn place_piece_at(&mut self, piece: Piece, pos: &Point2) {
        self.get_cell_mut(pos).piece = Some(piece);

        if let Some(effect) = piece.effect {
            for point in effect.range.reachable_points(pos, &self, &RangeContext::Area).iter() {
                self.get_cell_mut(point).effects.push(EffectKind::Protection);
            }
        }
    }

    fn get_cell_mut(&mut self, pos: &Point2) -> &mut Cell {
        &mut self.cells[pos.x as usize][pos.y as usize]
    }
    fn get_cell(&self, pos: &Point2) -> &Cell {
        &self.cells[pos.x as usize][pos.y as usize]
    }

    pub fn remove_piece_at(&mut self, pos: &Point2) {
        let piece = self.get_cell_mut(pos).piece.take()
            .expect(format!("Cannot remove: There is no piece on {:?}", pos).as_str());

        if let Some(effect) = piece.effect {
            for point in effect.range.reachable_points(pos, &self, &RangeContext::Area).iter() {
                let effects = &mut self.get_cell_mut(point).effects;
                if let Some(pos) = effects.iter_mut().position(|e| *e == EffectKind::Protection) {
                    effects.swap_remove(pos);
                }
            }
        }
    }

    /*pub(crate) fn move_piece(&mut self, from_x: u8, from_y: u8, to_x: u8, to_y: u8) {
        let piece = self.remove_piece(from_x, from_y);
        self.place_piece(piece, to_x, to_y);
    }
    fn move_piece_at(&mut self, piece: Piece, from_pos: &Point2, to_pos: &Point2) {
        self.remove_piece_at(from_pos);
        self.place_piece_at(piece, to_pos);
    }*/
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, SerBin, DeBin)]
pub struct Point2 {
    pub x: u8,
    pub y: u8,
}

impl Point2 {
    pub fn new(x: u8, y: u8) -> Self {
        Point2 { x, y }
    }
}

#[derive(Debug, PartialEq)]
pub enum PatternComponent {
    OwnPiece,
    Free,
    Any,
}

pub struct Pattern {
    pub components: Vec<Vec<PatternComponent>>,
    pub turn_into: PieceKind,
    pub new_piece_relative_position: Point2,
}

impl Pattern {
    pub fn all_patterns() -> [Pattern; 6] {
        [
            Pattern {
                components: vec![
                    vec![
                        PatternComponent::Any,
                        PatternComponent::Any,
                        PatternComponent::OwnPiece,
                        PatternComponent::Any,
                        PatternComponent::Any,
                    ],
                    vec![
                        PatternComponent::Any,
                        PatternComponent::OwnPiece,
                        PatternComponent::Free,
                        PatternComponent::OwnPiece,
                        PatternComponent::Any,
                    ],
                    vec![
                        PatternComponent::OwnPiece,
                        PatternComponent::Free,
                        PatternComponent::Free,
                        PatternComponent::Free,
                        PatternComponent::OwnPiece,
                    ],
                    vec![
                        PatternComponent::Any,
                        PatternComponent::OwnPiece,
                        PatternComponent::Free,
                        PatternComponent::OwnPiece,
                        PatternComponent::Any,
                    ],
                    vec![
                        PatternComponent::Any,
                        PatternComponent::Any,
                        PatternComponent::OwnPiece,
                        PatternComponent::Any,
                        PatternComponent::Any,
                    ],
                ],
                turn_into: PieceKind::Queen,
                new_piece_relative_position: Point2::new(2, 2),
            },
            Pattern {
                components: vec![
                    vec![
                        PatternComponent::Any,
                        PatternComponent::OwnPiece,
                        PatternComponent::Any,
                    ],
                    vec![
                        PatternComponent::OwnPiece,
                        PatternComponent::OwnPiece,
                        PatternComponent::OwnPiece,
                    ],
                    vec![
                        PatternComponent::Any,
                        PatternComponent::OwnPiece,
                        PatternComponent::Any,
                    ],
                ],
                turn_into: PieceKind::Cross,
                new_piece_relative_position: Point2::new(1, 1),
            },
            Pattern {
                components: vec![
                    vec![
                        PatternComponent::Free,
                        PatternComponent::Free,
                        PatternComponent::Free,
                    ],
                    vec![
                        PatternComponent::OwnPiece,
                        PatternComponent::OwnPiece,
                        PatternComponent::OwnPiece,
                    ],
                    vec![
                        PatternComponent::Free,
                        PatternComponent::Free,
                        PatternComponent::Free,
                    ],
                ],
                turn_into: PieceKind::HorizontalBar,
                new_piece_relative_position: Point2::new(1, 1),
            },
            Pattern {
                components: vec![
                    vec![
                        PatternComponent::Free,
                        PatternComponent::OwnPiece,
                        PatternComponent::Free,
                    ],
                    vec![
                        PatternComponent::Free,
                        PatternComponent::OwnPiece,
                        PatternComponent::Free,
                    ],
                    vec![
                        PatternComponent::Free,
                        PatternComponent::OwnPiece,
                        PatternComponent::Free,
                    ],
                ],
                turn_into: PieceKind::VerticalBar,
                new_piece_relative_position: Point2::new(1, 1),
            },
            Pattern {
                components: vec![
                    vec![
                        PatternComponent::OwnPiece,
                        PatternComponent::Any,
                        PatternComponent::OwnPiece,
                    ],
                    vec![
                        PatternComponent::Any,
                        PatternComponent::OwnPiece,
                        PatternComponent::Any,
                    ],
                    vec![
                        PatternComponent::OwnPiece,
                        PatternComponent::Any,
                        PatternComponent::OwnPiece,
                    ],
                ],
                turn_into: PieceKind::Sniper,
                new_piece_relative_position: Point2::new(1, 1),
            },
            Pattern {
                components: vec![
                    vec![
                        PatternComponent::Any,
                        PatternComponent::OwnPiece,
                        PatternComponent::Any,
                    ],
                    vec![
                        PatternComponent::OwnPiece,
                        PatternComponent::Free,
                        PatternComponent::OwnPiece,
                    ],
                    vec![
                        PatternComponent::Any,
                        PatternComponent::OwnPiece,
                        PatternComponent::Any,
                    ],
                ],
                turn_into: PieceKind::Castle,
                new_piece_relative_position: Point2::new(1, 1),
            },
        ]
    }
}
