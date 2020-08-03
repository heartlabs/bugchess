use amethyst::{
    core::math::{Point2},
    ecs::{Component, DenseVecStorage, Entity},
    renderer::palette::Srgba,
};

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

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct Cell {

}

#[derive(Component, Debug, Copy, Clone)]
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
    TargetOfHovered,
    Protected,
}