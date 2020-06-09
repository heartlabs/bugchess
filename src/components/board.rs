use amethyst::core::math::{Point2};
use amethyst::ecs::{Component, DenseVecStorage, Entity};
use amethyst::renderer::palette::Srgba;

use std::iter::successors;
use std::vec::IntoIter;
use ncollide3d::query::algorithms::gjk::directional_distance;

pub const BOARD_WIDTH: u8 = 5;
pub const BOARD_HEIGHT: u8 = 5;


#[derive(Debug, Copy, Clone)]
pub enum BoardEvent {
    Cell {
        x: u8,
        y: u8
    },
    Next,
    None,
}

impl Default for BoardEvent {
    fn default() -> BoardEvent {
        BoardEvent::None
    }
}

#[derive(Clone, Copy)]
pub struct Team {
    pub id: usize,
    pub color: Srgba,
    pub name: &'static str,
    pub lost: bool,
}

#[derive(Component, Clone, Copy)]
#[storage(DenseVecStorage)]
pub struct TeamAssignment {
    pub id: usize,
//    pub color: Srgba,
}

#[derive(Component, Clone, Copy)]
#[storage(DenseVecStorage)]
pub struct Piece {
    pub attack: bool
}

impl Piece {
    pub fn new() -> Piece {
        Piece {attack: true}
    }
}

#[derive(Component, Clone, Copy)]
#[storage(DenseVecStorage)]
pub struct Dying {

}
#[derive(Component, Clone, Copy)]
#[storage(DenseVecStorage)]
pub struct Exhausted {

}

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct Move {
    pub range: Range,
}

impl Move {
    pub fn new(direction: Direction, steps: u8) -> Move {
        Move {
            range: Range {direction, steps}
        }
    }
}

pub struct Range {
    pub direction: Direction,
    pub steps: u8,
}

impl Range {
    pub fn reaches(&self, from_x: u8, from_y: u8, to_x: u8, to_y: u8) -> bool {
        self.direction.reaches(from_x, from_y, to_x, to_y)
            && (from_y as i16 - to_y as i16).abs() as u8 <= self.steps
            && (from_x as i16 - to_x as i16).abs() as u8 <= self.steps
    }

    pub fn paths(&self, from_x: u8, from_y: u8) -> Box<dyn Iterator<Item=Box<dyn Iterator<Item=(i16,i16)>>>> {
        Box::new(self.direction.paths().into_iter().map(move |i| {
            let x: Box<Iterator<Item=(i16,i16)>> = Box::new(successors(
                Some((from_x as i16, from_y as i16)),
                move |&x| Some(i(x)))
                    .skip(1)
                    .take(self.steps as usize));
            x
        }).collect::<Vec<Box<dyn Iterator<Item=(i16,i16)>>>>().into_iter())

    }
}

impl Direction {
    fn reaches(&self, from_x: u8, from_y: u8, to_x: u8, to_y: u8) -> bool{
        match &self {
            Direction::Vertical => from_x == to_x,
            Direction::Horizontal => from_y == to_y,
            Direction::Diagonal => (from_y as i16 - to_y as i16).abs() == (from_x as i16 - to_x as i16).abs(),
            Direction::Straight => from_x == to_x || from_y == to_y,
            Direction::Star =>(from_y as i16 - to_y as i16).abs() == (from_x as i16 - to_x as i16).abs() || from_x == to_x || from_y == to_y,
            Direction::Anywhere => true
        }
    }

    fn paths(&self) -> Vec<Box<dyn Fn((i16,i16)) -> (i16,i16)>>{
        match &self {
            Direction::Vertical => vec![
                Box::new(|(x,y)| (x,y+1)),
                Box::new(|(x,y)| (x,y-1))
            ],
            Direction::Horizontal => vec![
                Box::new(|(x,y)| (x+1,y)),
                Box::new(|(x,y)| (x-1,y))
            ],
            Direction::Diagonal => vec![
                Box::new(|(x,y)| (x+1,y+1)),
                Box::new(|(x,y)| (x-1,y-1)),
                Box::new(|(x,y)| (x+1,y-1)),
                Box::new(|(x,y)| (x-1,y+1)),
            ],
            Direction::Straight => vec![
                Box::new(|(x,y)| (x+1,y)),
                Box::new(|(x,y)| (x-1,y)),
                Box::new(|(x,y)| (x,y+1)),
                Box::new(|(x,y)| (x,y-1))
            ],
            Direction::Star => vec![
                Box::new(|(x,y)| (x+1,y+1)),
                Box::new(|(x,y)| (x-1,y-1)),
                Box::new(|(x,y)| (x+1,y-1)),
                Box::new(|(x,y)| (x-1,y+1)),
                Box::new(|(x,y)| (x+1,y)),
                Box::new(|(x,y)| (x-1,y)),
                Box::new(|(x,y)| (x,y+1)),
                Box::new(|(x,y)| (x,y-1))
            ],
            Direction::Anywhere => Vec::new()
        }
    }
}

pub enum Direction {
    Vertical,
    Horizontal,
    Diagonal,
    Straight,
    Star,
    Anywhere
}

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct Cell {

}

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct BoardPosition {
    pub coords: Point2<u8>,
}

impl BoardPosition {
    pub fn new(x: u8, y: u8) -> BoardPosition {
        BoardPosition {
            coords: Point2::<u8>::new(x,y)
        }
    }
}

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct Target {
    possible_target_of: Vec<Entity>,
}

impl Target {
    pub fn new() -> Target {
        Target {
            possible_target_of: Vec::new(),
        }
    }

    pub fn clear(&mut self){
        self.possible_target_of.clear();
    }

    pub fn add(&mut self, entity: Entity){
        self.possible_target_of.push(entity);
    }

    pub fn is_possible_target_of(&self, entity: Entity) -> bool {
        self.possible_target_of.contains(&entity)
    }
}

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct Highlight {
    pub types: Vec<HighlightType>,
}

impl Highlight {
    pub fn new() -> Highlight {
        Highlight{types: Vec::new()}
    }
}

#[derive(Debug)]
pub enum HighlightType {
    Selected,
    Hovered,
    TargetOfSelected,
    TargetOfHovered
}

#[derive(Clone, Copy, Debug)]
pub enum PieceKind {
    Simple,
    HorizontalBar,
    VerticalBar,
    Cross,
}

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct TurnInto {
    pub kind: PieceKind,
}