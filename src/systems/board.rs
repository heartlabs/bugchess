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

    fn run(&mut self, (_cells, _activatables, _pieces, _sprites, _board, _entities, _transforms, _sprite_renders, _tints): Self::SystemData) {

    }
}
