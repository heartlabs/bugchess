use nanoserde::{DeJson, SerJson};
use std::fmt::Display;

pub mod board;
pub mod game;
pub mod pattern;
pub mod piece;
pub mod ranges;

pub type GameResult<T> = Result<T, GameError>;

#[derive(Debug, Clone)]
pub struct GameError {
    description: String,
}

impl GameError {
    pub fn new(description: String) -> Self {
        Self { description }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, SerJson, DeJson)]
pub struct Point2 {
    pub x: u8,
    pub y: u8,
}

impl From<Point2> for (u8, u8) {
    fn from(point: Point2) -> (u8, u8) {
        (point.x, point.y)
    }
}

impl Point2 {
    pub fn new(x: u8, y: u8) -> Self {
        Point2 { x, y }
    }
}

impl From<(u8, u8)> for Point2 {
    fn from((x, y): (u8, u8)) -> Self {
        Point2 { x, y }
    }
}

impl Display for Point2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
