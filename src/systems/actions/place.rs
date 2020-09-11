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
    pub kind: Option<PieceKind>,
}

impl Place {
    // This places a piece (again) that is already initialized
    pub fn new(entity: Entity, pos: BoardPosition) -> Box<Self> {
        Box::new(Place {
            entity,
            pos,
            kind: None
        })
    }

    // This initializes the piece when it is created the first time
    pub fn introduce(entity: Entity, pos: BoardPosition, kind: PieceKind) -> Box<Self> {
        Box::new(Place {
            entity,
            pos,
            kind: Some(kind)
        })
    }
}

impl<'a> System<'a> for Place {
    type SystemData = (
        WriteExpect<'a, Board>,
        WriteStorage<'a, Piece>,
        WriteStorage<'a, BoardPosition>,
        WriteStorage<'a, TurnInto>,
    );

    fn run(&mut self, (mut board, mut pieces, mut positions, mut turn_intos): Self::SystemData) {
        board.place_piece_at(self.entity, &self.pos);

        println!("Placed new piece");
        positions.insert(self.entity, self.pos);

        let mut piece = pieces.get_mut(self.entity).unwrap();
        piece.exists = true;

        if let Some(kind) = self.kind {
            turn_intos.insert(self.entity, TurnInto{kind});
        }
    }
}

impl Action for Place {
    fn get_anti_action(&self) -> Box<dyn Action + Send + Sync> {
        Box::new(Remove {
            entity: self.entity,
            pos: self.pos,
        })
    }
}

impl HasRunNow for Place {
    fn get_run_now<'a>(&self) -> Box<dyn RunNow<'a> + 'a> {
        Box::new(Place {entity: self.entity, pos: self.pos, kind: self.kind})
    }
}