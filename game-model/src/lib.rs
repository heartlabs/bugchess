use std::fmt::Display;
use nanoserde::{DeBin, SerBin};

pub mod board;
pub mod game;
pub mod piece;
pub mod ranges;
pub mod pattern;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, SerBin, DeBin)]
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
