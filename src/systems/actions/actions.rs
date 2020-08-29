use amethyst::{
    ecs::{Entity, System, WriteExpect, RunNow},
    prelude::World,
};
use crate::{
    resources::board::Board,
};

pub trait Action : HasRunNow {
    fn get_anti_action(&self) -> Box<dyn Action>;
}

pub trait HasRunNow {
    fn get_run_now<'a> (&self) -> Box<dyn RunNow<'a>  + 'a>;
}

#[derive(Copy,Clone)]
pub struct AddUnusedPiece {
    entity: Entity,
    remove: bool
}

pub struct CompoundAction {
    components: Vec<Box<dyn Action>>,
}

pub struct CompoundRunnable<'a> {
    components: Vec<Box<dyn RunNow<'a> + 'a>>,
}

impl Action for CompoundAction {
    fn get_anti_action(&self) -> Box<dyn Action> {
        let anti_components: Vec<Box<dyn Action>> = self.components.iter().map(|a| a.get_anti_action()).collect();

        Box::new(CompoundAction {
            components: anti_components
        })
    }
}

impl HasRunNow for CompoundAction {
    fn get_run_now<'a>(&self) -> Box<dyn RunNow<'a> + 'a> {
        Box::new(CompoundRunnable{ components: self.components.iter().map(|a| a.get_run_now()).collect() })
    }
}

impl<'a> RunNow<'a> for CompoundRunnable<'a> {
    fn run_now(&mut self, world: &'a World) {
        self.components.iter_mut().for_each(|a|a.run_now(world));
    }
    fn setup(&mut self, world: &mut World) {
        self.components.iter_mut().for_each(|a|a.setup(world));
    }
}

impl Action for AddUnusedPiece {

    fn get_anti_action(&self) -> Box<dyn Action> {
        Box::new(AddUnusedPiece {
            entity: self.entity,
            remove: !self.remove,
        })
    }
}

impl HasRunNow for AddUnusedPiece {
    fn get_run_now<'a>(&self) -> Box<dyn RunNow<'a> + 'a> {
        Box::new(AddUnusedPiece { entity: self.entity, remove: self.remove })
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
