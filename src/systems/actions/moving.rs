use amethyst::ecs::{System, Entity, WriteExpect};

use crate::{
    components::board::BoardPosition,
    resources::board::Board,
    systems::actions::{
        actions::Action,
    },
};
use amethyst::core::ecs::{WriteStorage, RunNow};
use crate::systems::actions::actions::HasRunNow;

pub struct Move {
    entity: Entity,
    from_pos: BoardPosition,
    to_pos: BoardPosition,
}

impl Move {
    pub fn new(entity: Entity, from_pos: BoardPosition, to_pos: BoardPosition) -> Box<Self> {
        Box::new(Move {
            entity,
            from_pos,
            to_pos
        })
    }
}

impl<'a> System<'a> for Move {
    type SystemData = (
        WriteExpect<'a, Board>,
        WriteStorage<'a, BoardPosition>,
    );

    fn run(&mut self, (mut board, mut positions): Self::SystemData) {
        let mut pos = positions.get_mut(self.entity).unwrap();

        pos.coords = self.to_pos.coords.clone();

        board.move_piece_at(self.entity, &self.from_pos, &self.to_pos);
    }
}
impl Action for Move {

    fn get_anti_action(&self) -> Box<dyn Action + Send + Sync> {
        Box::new(Move {
            entity: self.entity,
            from_pos: self.to_pos,
            to_pos: self.from_pos
        })
    }
}


impl HasRunNow for Move {
    fn get_run_now<'a>(&self) -> Box<dyn RunNow<'a> + 'a> {
        Move::new(self.entity, self.from_pos, self.to_pos)
    }
}