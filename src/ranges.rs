use crate::*;
use nanoserde::{DeBin, SerBin};
use std::{collections::HashSet, iter::successors};

#[derive(Debug, Copy, Clone, SerBin, DeBin)]
pub struct Range {
    pub direction: Direction,
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

pub enum RangeContext {
    Moving(Piece),
    Special(Piece),
    Area,
}

impl RangeContext {
    pub fn should_proceed(&self, point: &Point2, board: &Board) -> bool {
        match self {
            RangeContext::Moving(_) => board.get_piece_at(point).is_none(),
            RangeContext::Special(_) => {
                board.get_piece_at(point).is_none()
                    && !board.has_effect_at(&EffectKind::Protection, point)
            }
            RangeContext::Area => true,
        }
    }

    pub fn should_include(&self, point: &Point2, board: &Board) -> bool {
        match self {
            RangeContext::Moving(piece) => {
                if let Some(target_piece) = board.get_piece_at(point) {
                    return (!target_piece.shield || piece.pierce)
                        && target_piece.team_id != piece.team_id;
                }
                true
            }
            RangeContext::Special(piece) => {
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
    pub fn new_unlimited(direction: Direction) -> Range {
        Range {
            direction,
            steps: 255,
            jumps: false,
            include_self: false,
        }
    }

    pub fn new(direction: Direction, steps: u8) -> Range {
        Range {
            direction,
            steps,
            jumps: false,
            include_self: false,
        }
    }

    pub fn anywhere() -> Range {
        Range {
            direction: Direction::Anywhere,
            steps: 255,
            jumps: true,
            include_self: false,
        }
    }

    /*    pub fn reaches(&self, from: &Point2, to: &Point2) -> bool {
        self.direction.reaches(from.x, from.y, to.x, to.y)
            && (from.y as i16 - to.y as i16).abs() as u8 <= self.steps
            && (from.x as i16 - to.x as i16).abs() as u8 <= self.steps
    }*/

    pub fn paths(
        &self,
        from_x: u8,
        from_y: u8,
    ) -> Box<dyn Iterator<Item = Box<dyn Iterator<Item = (i16, i16)>>>> {
        if self.direction == Direction::Anywhere {
            // TODO: This only works if jump=true and steps=max
            let row_iter = (0..255 as i16).map(move |x| {
                Box::new((0..255 as i16).map(move |y| (x, y)))
                    as Box<dyn Iterator<Item = (i16, i16)>>
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

    pub fn reachable_points(
        &self,
        from_point: &Point2,
        board: &Board,
        range_context: &RangeContext,
    ) -> HashSet<Point2> {
        let mut cells = HashSet::new();
        for direction in self.paths(from_point.x, from_point.y) {
            for (x_i16, y_i16) in direction {
                let point = Point2::new(x_i16 as u8, y_i16 as u8);
                if board.has_cell(&point) {
                    if range_context.should_include(&point, board) {
                        cells.insert(point);
                    }
                    if !self.jumps && !range_context.should_proceed(&point, board) {
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
