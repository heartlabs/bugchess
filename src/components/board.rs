use amethyst::core::math::{Point2};
use amethyst::ecs::{Component, DenseVecStorage, Entity};
use amethyst::renderer::palette::Srgba;

use std::iter::successors;
use std::vec::IntoIter;
use ncollide3d::query::algorithms::gjk::directional_distance;
use crate::resources::board::Board;

pub const BOARD_WIDTH: u8 = 8;
pub const BOARD_HEIGHT: u8 = 8;


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
            range: Range::new(direction, steps)
        }
    }
}

#[derive(Debug)]
pub struct Range {
    pub direction: Direction,
    pub steps: u8,
    pub jumps: bool,
    pub include_self: bool,
}

impl Range {
    pub fn new_unlimited(direction: Direction) -> Range {
        Range {direction, steps: 255, jumps: false, include_self: false}
    }

    pub fn new(direction: Direction, steps: u8) -> Range {
        Range {direction, steps, jumps: false, include_self: false}
    }

    pub fn anywhere() -> Range {
        Range {direction: Direction::Anywhere, steps: 255, jumps: true, include_self: false}
    }

    pub fn reaches(&self, from_x: u8, from_y: u8, to_x: u8, to_y: u8) -> bool {
        self.direction.reaches(from_x, from_y, to_x, to_y)
            && (from_y as i16 - to_y as i16).abs() as u8 <= self.steps
            && (from_x as i16 - to_x as i16).abs() as u8 <= self.steps
    }

    pub fn paths(&self, from_x: u8, from_y: u8) -> Box<dyn Iterator<Item=Box<dyn Iterator<Item=(i16,i16)>>>> {
        if self.direction == Direction::Anywhere { // TODO: This only works if jump=true and steps=max
            let row_iter = (0..255 as i16).map(move |x| {
                Box::new((0..255 as i16).map(move |y| (x, y))) as Box<Iterator<Item=(i16, i16)>>
            });
            return Box::new(row_iter);
        }

        let mut vec = self.direction.paths().into_iter().map(move |i| {
            let x: Box<Iterator<Item=(i16,i16)>> = Box::new(successors(
                Some((from_x as i16, from_y as i16)),
                move |&x| Some(i(x)))
                .skip(1)
                .take(self.steps as usize));
            x
        }).collect::<Vec<Box<dyn Iterator<Item=(i16,i16)>>>>();

        if self.include_self {
            vec.push(Box::new(Some((from_x as i16,from_y as i16)).into_iter()));
        }

        Box::new(vec.into_iter())
    }

    pub fn for_each<F>(&self, from_x: u8, from_y: u8, board: &Board, mut perform: F) where F: FnMut(u8,u8,Entity) -> bool {
        for direction in self.paths(from_x, from_y) {
            for (x_i16,y_i16) in direction {
                let (x,y) = (x_i16 as u8, y_i16 as u8);
                if let Some(cell) = board.get_cell_safely(x_i16, y_i16) {
                    let proceed = perform(x,y,cell);
                    if (!self.jumps && (board.get_piece(x, y).is_some() || !proceed)) {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
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

#[derive(Debug,PartialEq)]
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
    possible_special_target_of: Vec<Entity>,
    pub protected: bool,
}

impl Target {
    pub fn new() -> Target {
        Target {
            possible_target_of: Vec::new(),
            possible_special_target_of: Vec::new(),
            protected: false,
        }
    }

    pub fn clear(&mut self){
        self.possible_special_target_of.clear();
        self.possible_target_of.clear();
        self.protected = false;
    }

    pub fn add(&mut self, entity: Entity){
        self.possible_target_of.push(entity);
    }

    pub fn add_special(&mut self, entity: Entity){
        self.possible_special_target_of.push(entity);
    }

    pub fn is_possible_target_of(&self, entity: Entity) -> bool {
        self.possible_target_of.contains(&entity)
    }

    pub fn is_possible_special_target_of(&self, entity: Entity) -> bool {
        self.possible_special_target_of.contains(&entity)
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
    Queen,
    Castle,
    Sniper
}

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct TurnInto {
    pub kind: PieceKind,
}

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct ActivatablePower {
    pub kind: Power,
    pub range: Range,
}

#[derive(Debug)]
pub enum Power {
    Blast,
    TargetedShoot
}

#[derive(Debug,PartialEq)]
pub enum EffectKind {
    Protection,
}

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct Effect {
    pub kind: EffectKind,
    pub range: Range,
}