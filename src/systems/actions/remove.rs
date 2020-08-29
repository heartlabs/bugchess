use amethyst::ecs::{System, Entity, WriteExpect};

use crate::{
    components::board::BoardPosition,
    resources::board::Board,
    systems::actions::{
        actions::Action,
        place::Place,
    },
};

pub struct Remove {
    pub entity: Entity,
    pub pos: BoardPosition
}

impl<'a> System<'a> for Remove {
    type SystemData = (WriteExpect<'a, Board>);

    fn run(&mut self, (mut board): Self::SystemData) {
        board.remove_piece_at(&self.pos);
    }
}

// impl<'a> Action<'a> for Remove {
//     fn get_anti_action(&self) -> Box<dyn Action<'a> + 'a> {
//         Box::new(Place {
//             entity: self.entity,
//             pos: self.pos
//         })
//     }
// }