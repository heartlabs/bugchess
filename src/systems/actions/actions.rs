use amethyst::{
    ecs::{Entity, System, WriteExpect, RunNow},
    prelude::World,
    renderer::SpriteRender,
    core::{
        math::{Vector3},
        ecs::{ReadStorage, WriteStorage},
        Transform
    },
};
use crate::{
    resources::board::Board,
    components::piece::Piece,
};
use crate::states::load::Sprites;

pub trait Action : HasRunNow {
    fn get_anti_action(&self) -> Box<dyn Action + Send + Sync>;
    fn finalize(&self, world: &World) {
        // Do nothing
    }
}

pub trait HasRunNow {
    fn get_run_now<'a> (&self) -> Box<dyn RunNow<'a>  + 'a>;
}

pub trait ProducesActions {
    fn get_actions(&self) -> Vec<Box<dyn Action + Send + Sync>> {
        Vec::new()
    }
}

#[derive(Copy,Clone)]
pub struct AddUnusedPiece {
    entity: Entity,
    remove: bool
}

impl AddUnusedPiece {
    pub fn add(entity: Entity) -> AddUnusedPiece {
        AddUnusedPiece {
            entity,
            remove: false
        }
    }

    pub fn remove(entity: Entity) -> AddUnusedPiece {
        AddUnusedPiece {
            entity,
            remove: true
        }
    }
}

pub struct CompoundAction {
    components: Vec<Box<dyn Action + Send + Sync>>,
}

// TODO: Why isn't this derived automatically?
unsafe impl Send for CompoundAction {}
unsafe impl Sync for CompoundAction {}

impl CompoundAction {
    pub fn new() -> CompoundAction {
        CompoundAction { components: Vec::new() }
    }

    pub fn add(&mut self, action: Box<dyn Action + Send + Sync>) {
        self.components.push(action);
    }

    pub fn transfer_content_to(&mut self, target: &mut CompoundAction) {
        for action in self.components.drain(..) {
            target.add(action);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
}

pub struct CompoundRunnable<'a> {
    components: Vec<Box<dyn RunNow<'a> + 'a>>,
}

impl Action for CompoundAction {
    fn get_anti_action(&self) -> Box<dyn Action + Send + Sync> {
        let anti_components: Vec<Box<dyn Action + Send + Sync>> = self.components.iter()
            .rev() // Actions need to be undone in reverse order
            .map(|a| a.get_anti_action())
            .collect();

        Box::new(CompoundAction {
            components: anti_components
        })
    }

    fn finalize(&self, world: &World) {
        self.components.iter().for_each(|a| a.finalize(world));
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

    fn get_anti_action(&self) -> Box<dyn Action + Send + Sync> {
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
    type SystemData = (
        WriteExpect<'a, Board>,
        WriteExpect<'a, Sprites>,
        WriteStorage<'a, SpriteRender>,
        ReadStorage<'a, Piece>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, (mut board, sprites, mut sprite_renders, pieces, mut transforms): Self::SystemData) {
        if self.remove {
            println!("Removing unused piece {:?}", self.entity);
            board.discard_unused_piece(self.entity);
            transforms.remove(self.entity);
        }else{


            let mut transform = Transform::default();
            let piece = pieces.get(self.entity).unwrap();

            // 2 rows with 10 pieces each per team
            let row: usize = piece.team_id * 2 + board.num_unused_pieces_of(piece.team_id)/10;
            let column: usize = board.num_unused_pieces_of(piece.team_id)%10;

            let x_offset = 650;
            let y_offset = 100;

            let piece_width = 32;

            let screen_x = (x_offset + column * piece_width) as f32;
            let screen_y = (y_offset + row * piece_width) as f32;
            //println!("New unused piece at {}:{}", screen_x, screen_y);

            transform.set_translation_xyz(screen_x, screen_y, 0.1);
            transform.set_scale(Vector3::new(0.5,0.5,1.));
            transforms.insert(self.entity, transform);

            sprite_renders.insert(self.entity, sprites.sprite_piece.clone());

            board.add_unused_piece_for(self.entity, piece.team_id);

            println!("Adding unused piece {:?} at {}:{}", self.entity, screen_x, screen_y);
        }
    }
}
