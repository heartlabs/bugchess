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
use crate::components::piece::Piece;

pub struct Exhaust {
    entity: Entity,
    moved: bool,
    is_anti: bool
}

impl Exhaust {
    pub fn moved(entity: Entity) -> Box<Self> {
        Box::new(Exhaust {
            entity,
            moved: true,
            is_anti: false
        })
    }
    pub fn special(entity: Entity) -> Box<Self> {
        Box::new(Exhaust {
            entity,
            moved: false,
            is_anti: false
        })
    }
}

impl<'a> System<'a> for Exhaust {
    type SystemData = (
        WriteStorage<'a, Piece>,
    );

    fn run(&mut self, (mut pieces,): Self::SystemData) {
        let mut piece = pieces.get_mut(self.entity).unwrap();
        let mut exhaustion = &mut piece.exhaustion;

        if self.is_anti {
            if self.moved {
                exhaustion.undo_move();
            } else {
                exhaustion.undo_attack();
            }
        } else {
            if self.moved {
                exhaustion.on_move();
            } else {
                exhaustion.on_attack();
            }
        }
    }
}
impl Action for Exhaust {

    fn get_anti_action(&self) -> Box<dyn Action + Send + Sync> {
        Box::new(Exhaust {
            entity: self.entity,
            moved: self.moved,
            is_anti: !self.is_anti
        })
    }
}


impl HasRunNow for Exhaust {
    fn get_run_now<'a>(&self) -> Box<dyn RunNow<'a> + 'a> {
        Box::new(Exhaust {
            entity: self.entity,
            moved: self.moved,
            is_anti: self.is_anti
        })
    }
}