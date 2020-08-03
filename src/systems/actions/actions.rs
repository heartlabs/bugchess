use amethyst::{
    ecs::{Entity, System, WriteExpect, RunNow},
    prelude::World,
};
use crate::{
    resources::board::Board,
};

pub trait Action<'a>: RunNow<'a> {
    fn get_anti_action(&self) -> Box<dyn Action<'a>  + 'a>;
}




pub struct AddUnusedPiece {
    entity: Entity,
    remove: bool
}

pub struct CompoundAction<'a> {
    components: Vec<Box<dyn Action<'a> + 'a>>,
}

impl<'a, 'b> RunNow<'a> for CompoundAction<'a> {
    fn run_now(&mut self, world: &'a World) {
        self.components.iter_mut().for_each(|a|a.run_now(world));
    }
    fn setup(&mut self, world: &mut World) {
        self.components.iter_mut().for_each(|a|a.setup(world));
    }
}

impl<'a> Action<'a> for CompoundAction<'a> {
    fn get_anti_action(&self) -> Box<dyn Action<'a> + 'a> {
        let anti_components: Vec<Box<dyn Action<'a> + 'a>> = self.components.iter().map(|a| a.get_anti_action()).collect();

        Box::new(CompoundAction {
            components: anti_components
        })
    }
}

impl<'a> System<'a> for AddUnusedPiece {
    type SystemData = (WriteExpect<'a, Board>,);

    fn run(&mut self, (mut board,): Self::SystemData) {
        if self.remove {
            board.discard_unused_piece();
        }else{
            board.add_unused_piece(self.entity);
        }
    }
}

impl<'a> Action<'a> for AddUnusedPiece {

    fn get_anti_action(&self) -> Box<dyn Action<'a> + 'a> {
        Box::new(AddUnusedPiece {
            entity: self.entity,
            remove: !self.remove,
        })
    }
}

