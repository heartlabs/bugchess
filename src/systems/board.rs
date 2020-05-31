use amethyst::{
    core::Transform,
    ecs::{Join, ReadStorage, System, WriteStorage, Write, ReadExpect, WriteExpect, Entities},
    renderer:: {
        SpriteRender,
        resources::Tint,
        palette::Srgba,
    },
    shrev::EventChannel,
    core::math::Vector3,
};
use crate::components::{Cell, Activatable, Piece, Board};
use crate::components::board::Team;
use std::borrow::BorrowMut;
use crate::states::load::Sprites;

pub struct BoardSystem;

impl<'a> System<'a> for BoardSystem {
    type SystemData = (
        ReadStorage<'a, Cell>,
        WriteStorage<'a, Activatable>,
        WriteStorage<'a, Piece>,
        ReadExpect<'a, Sprites>,
        WriteExpect<'a, Board>,
        Entities<'a>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, SpriteRender>,
        WriteStorage<'a, Tint>,
    );

    fn run(&mut self, (cells, mut activatables, mut pieces, sprites, mut board, entities, mut transforms, mut sprite_renders, mut tints): Self::SystemData) {

    }
}
