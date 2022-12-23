use indexmap::IndexSet;
use nanoserde::{DeBin, SerBin};
use std::{collections::HashSet, iter::successors};

use crate::{board::*, piece::*};

#[derive(Debug, Copy, Clone, SerBin, DeBin)]
pub struct Range {
    pub direction: Direction,
    pub context: RangeContext,
    pub steps: u8,
    pub jumps: bool,
    pub include_self: bool,
}

#[derive(Debug, PartialEq, Copy, Clone, SerBin, DeBin)]
pub enum Direction {
    Vertical,
    Horizontal,
    Diagonal,
    Straight,
    Star,
    Anywhere,
}

impl Direction {
    fn reaches(&self, from_x: u8, from_y: u8, to_x: u8, to_y: u8) -> bool {
        match &self {
            Direction::Vertical => from_x == to_x,
            Direction::Horizontal => from_y == to_y,
            Direction::Diagonal => {
                (from_y as i16 - to_y as i16).abs() == (from_x as i16 - to_x as i16).abs()
            }
            Direction::Straight => from_x == to_x || from_y == to_y,
            Direction::Star => {
                (from_y as i16 - to_y as i16).abs() == (from_x as i16 - to_x as i16).abs()
                    || from_x == to_x
                    || from_y == to_y
            }
            Direction::Anywhere => true,
        }
    }

    fn paths(&self) -> Vec<Box<dyn Fn((i16, i16)) -> (i16, i16)>> {
        match &self {
            Direction::Vertical => {
                vec![Box::new(|(x, y)| (x, y + 1)), Box::new(|(x, y)| (x, y - 1))]
            }
            Direction::Horizontal => {
                vec![Box::new(|(x, y)| (x + 1, y)), Box::new(|(x, y)| (x - 1, y))]
            }
            Direction::Diagonal => vec![
                Box::new(|(x, y)| (x + 1, y + 1)),
                Box::new(|(x, y)| (x - 1, y - 1)),
                Box::new(|(x, y)| (x + 1, y - 1)),
                Box::new(|(x, y)| (x - 1, y + 1)),
            ],
            Direction::Straight => vec![
                Box::new(|(x, y)| (x + 1, y)),
                Box::new(|(x, y)| (x - 1, y)),
                Box::new(|(x, y)| (x, y + 1)),
                Box::new(|(x, y)| (x, y - 1)),
            ],
            Direction::Star => vec![
                Box::new(|(x, y)| (x + 1, y + 1)),
                Box::new(|(x, y)| (x - 1, y - 1)),
                Box::new(|(x, y)| (x + 1, y - 1)),
                Box::new(|(x, y)| (x - 1, y + 1)),
                Box::new(|(x, y)| (x + 1, y)),
                Box::new(|(x, y)| (x - 1, y)),
                Box::new(|(x, y)| (x, y + 1)),
                Box::new(|(x, y)| (x, y - 1)),
            ],
            Direction::Anywhere => Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone, SerBin, DeBin)]
pub enum RangeContext {
    Moving,
    Special,
    Area,
}

impl RangeContext {
    pub fn should_proceed(&self, point: &Point2, board: &Board) -> bool {
        match self {
            RangeContext::Moving => board.get_piece_at(point).is_none(),
            RangeContext::Special => {
                board.get_piece_at(point).is_none()
                    && !board.has_effect_at(&EffectKind::Protection, point)
            }
            RangeContext::Area => true,
        }
    }

    pub fn should_include(&self, piece: &Piece, point: &Point2, board: &Board) -> bool {
        match self {
            RangeContext::Moving => {
                if let Some(target_piece) = board.get_piece_at(point) {
                    return (!target_piece.shield || piece.pierce)
                        && target_piece.team_id != piece.team_id;
                }
                true
            }
            RangeContext::Special => {
                if let Some(target_piece) = board.get_piece_at(point) {
                    return target_piece.team_id != piece.team_id
                        && !board.has_effect_at(&EffectKind::Protection, point);
                }

                false
            }
            RangeContext::Area => true,
        }
    }
}

impl Range {
    pub fn new_unlimited(direction: Direction, context: RangeContext) -> Range {
        Range {
            direction,
            context,
            steps: 255,
            jumps: false,
            include_self: false,
        }
    }

    pub fn new_moving(direction: Direction, steps: u8) -> Range {
        Range {
            direction,
            context: RangeContext::Moving,
            steps,
            jumps: false,
            include_self: false,
        }
    }

    pub fn paths(
        &self,
        from_x: u8,
        from_y: u8,
    ) -> Box<dyn Iterator<Item = Box<dyn Iterator<Item = (i16, i16)>>>> {
        if self.direction == Direction::Anywhere {
            let row_iter = (0..255_i16).flat_map(move |x| {
                (0..255_i16).map(move |y| {
                    Box::new(Some((x, y)).into_iter()) as Box<dyn Iterator<Item = (i16, i16)>>
                })
            });
            return Box::new(row_iter);
        }

        let mut vec = self
            .direction
            .paths()
            .into_iter()
            .map(move |i| {
                let x: Box<dyn Iterator<Item = (i16, i16)>> = Box::new(
                    successors(Some((from_x as i16, from_y as i16)), move |&x| Some(i(x)))
                        .skip(1)
                        .take(self.steps as usize),
                );
                x
            })
            .collect::<Vec<Box<dyn Iterator<Item = (i16, i16)>>>>();

        if self.include_self {
            vec.push(Box::new(Some((from_x as i16, from_y as i16)).into_iter()));
        }

        Box::new(vec.into_iter())
    }

    pub fn reachable_points(&self, from_point: &Point2, board: &Board) -> IndexSet<Point2> {
        let piece = board
            .get_piece_at(from_point)
            .expect(format!("No piece at {:?}", from_point).as_str());

        self.reachable_points_for_piece(from_point, piece, board)
    }
    
    pub fn reachable_points_for_piece(
        &self,
        from_point: &Point2,
        piece: &Piece,
        board: &Board,
    ) -> IndexSet<Point2> {
        let mut cells = IndexSet::new();
        for direction in self.paths(from_point.x, from_point.y) {
            for (x_i16, y_i16) in direction {
                let point = Point2::new(x_i16 as u8, y_i16 as u8);
                if board.has_cell(&point) {
                    if self.context.should_include(piece, &point, board) {
                        cells.insert(point);
                    }
                    if !self.jumps && !self.context.should_proceed(&point, board) {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        cells
    }
}
