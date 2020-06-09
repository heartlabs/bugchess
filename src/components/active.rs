use amethyst::ecs::{Component, DenseVecStorage};
use crate::components::board::BoardEvent;

#[derive(Component, Default, Debug)]
#[storage(DenseVecStorage)]
pub struct Activatable {
    pub event: BoardEvent,
}

#[derive(Component, Default, Debug)]
#[storage(DenseVecStorage)]
pub struct Selected {

}

#[derive(Component, Default, Debug)]
#[storage(DenseVecStorage)]
pub struct Hovered {

}