use amethyst::ecs::{System, Entity, WriteExpect};

use crate::{
    components::board::BoardPosition,
    resources::board::Board,
    systems::actions::{
        actions::Action,
    },
};
pub struct Move {
    entity: Entity,
    from_pos: BoardPosition,
    to_pos: BoardPosition,
}

impl<'a> System<'a> for Move {
    type SystemData = (WriteExpect<'a, Board>,);

    fn run(&mut self, (mut board,): Self::SystemData) {
        board.move_piece_at(self.entity, &self.from_pos, &self.to_pos);
    }
}
impl<'a> Action<'a> for Move {

    fn get_anti_action(&self) -> Box<dyn Action<'a> + 'a> {
        Box::new(Move {
            entity: self.entity,
            from_pos: self.to_pos,
            to_pos: self.from_pos
        })
    }
}