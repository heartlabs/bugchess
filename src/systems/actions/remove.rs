use amethyst::ecs::{System, Entity, WriteExpect};

use crate::{
    components::board::BoardPosition,
    resources::board::Board,
    systems::actions::{
        actions::Action,
        place::Place,
    },
};
use crate::systems::actions::actions::HasRunNow;
use amethyst::core::ecs::{RunNow, Entities, WriteStorage, World};
use crate::components::piece::{PieceKind, Piece};
use amethyst::core::Transform;
use amethyst::prelude::*;

pub struct Remove {
    pub entity: Entity,
    pub pos: BoardPosition,
}
impl Remove {
    pub fn new(entity: Entity, pos: BoardPosition) -> Box<Self> {
        Box::new(Remove {
            entity,
            pos,
        })
    }
}
impl<'a> System<'a> for Remove {
    type SystemData = (
        WriteExpect<'a, Board>,
        WriteStorage<'a, Piece>,
        WriteStorage<'a, BoardPosition>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, (mut board, mut pieces, mut positions, mut transforms): Self::SystemData) {
        board.remove_piece_at(&self.pos);

        transforms.remove(self.entity);
        positions.remove(self.entity);

        let mut piece = pieces.get_mut(self.entity).unwrap();
        piece.exists = false;
    }
}

impl Action for Remove {
    fn get_anti_action(&self) -> Box<dyn Action + Send + Sync> {
        Place::new(self.entity, self.pos)
    }

    fn finalize(&self, world: &World) {
        println!("Deleted entity {:?}", self.entity);
        world.entities_mut().delete(self.entity);
    }
}

impl HasRunNow for Remove {
    fn get_run_now<'a>(&self) -> Box<dyn RunNow<'a> + 'a> {
        Remove::new(self.entity, self.pos)
    }
}