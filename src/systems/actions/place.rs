use amethyst::ecs::{System, Entity, WriteExpect};

use crate::{
    components::board::BoardPosition,
    resources::board::Board,
    systems::actions::{
        actions::Action,
        remove::Remove,
    },
};

pub struct Place {
    pub entity: Entity,
    pub pos: BoardPosition
}

impl<'a> System<'a> for Place {
    type SystemData = (WriteExpect<'a, Board>,);

    fn run(&mut self, (mut board,): Self::SystemData) {
        board.place_piece_at(self.entity, &self.pos);
    }
}
impl<'a> Action<'a> for Place {
    fn get_anti_action(&self) -> Box<dyn Action<'a> + 'a> {
        Box::new(Remove {
            entity: self.entity,
            pos: self.pos
        })
    }
}