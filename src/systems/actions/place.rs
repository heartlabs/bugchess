use amethyst::ecs::{System, Entity, WriteExpect};

use crate::{
    components::board::BoardPosition,
    resources::board::Board,
    systems::actions::{
        actions::Action,
        remove::Remove,
    },
};
use crate::systems::actions::actions::HasRunNow;
use amethyst::core::ecs::{RunNow, WriteStorage};
use crate::components::piece::{TurnInto, PieceKind, Piece};

pub struct Place {
    pub entity: Entity,
    pub pos: BoardPosition,
    pub kind: PieceKind,
}

impl Place {
    pub fn new(entity: Entity, pos: BoardPosition, kind: PieceKind) -> Place {
        Place {
            entity,
            pos,
            kind
        }
    }
}

impl<'a> System<'a> for Place {
    type SystemData = (
        WriteExpect<'a, Board>,
        WriteStorage<'a, Piece>,
        WriteStorage<'a, BoardPosition>,
        WriteStorage<'a, TurnInto>,
    );

    fn run(&mut self, (mut _board, mut pieces, mut positions, mut turn_intos): Self::SystemData) {
        //board.place_piece_at(self.entity, &self.pos); This happens in some system actually
        println!("Placed new piece");
        positions.insert(self.entity, self.pos);
        turn_intos.insert(self.entity, TurnInto{kind: self.kind});

        let mut piece = pieces.get_mut(self.entity).unwrap();
        piece.exists = true;
    }
}

impl Action for Place {
    fn get_anti_action(&self) -> Box<dyn Action + Send + Sync> {
        Box::new(Remove {
            entity: self.entity,
            pos: self.pos,
            kind: self.kind
        })
    }
}

impl HasRunNow for Place {
    fn get_run_now<'a>(&self) -> Box<dyn RunNow<'a> + 'a> {
        Box::new(Place {entity: self.entity, pos: self.pos, kind: self.kind })
    }
}