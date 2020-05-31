use amethyst::ecs::{Component, DenseVecStorage};
use crate::components::board::BoardEvent;

#[derive(Component, Default)]
#[storage(DenseVecStorage)]
pub struct Activatable {
    pub active: bool,
    pub event: BoardEvent,
}
