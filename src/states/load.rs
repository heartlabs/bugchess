use amethyst::{
    assets::{AssetStorage, Loader},
    core::{
        transform::Transform,
        math::{Vector3},
    },
    ecs::{Entity,RunNow},
    input::{is_close_requested, is_key_down, VirtualKeyCode},
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture,
               palette::Srgba,
               resources::Tint},
    window::ScreenDimensions,
    ui::{UiCreator, UiFinder},
};

use ncollide3d::shape::Cuboid;

use crate::{
    components::{
        Activatable, Bounded, Mouse, Cell,
        board::{BoardPosition, Target, Highlight, BOARD_WIDTH, BOARD_HEIGHT, BoardEvent, Team},
        piece::{Piece, PieceKind, TurnInto, Effect}
    },
    resources::board::Board,
    states::{
        next_turn::{NextTurnState, TurnCounter},
    },

};
use crate::systems::actions::common::UpdateUi;
use crate::systems::actions::during_turn::{InitNewPieces, MergePiecePatterns, UpdateTargets};
use crate::systems::actions::actions::{Action, HasRunNow, CompoundAction};
use std::collections::VecDeque;
use crate::systems::actions::place::Place;

pub struct LoadingState;

#[derive(Clone)]
pub struct Sprites {
    pub sprite_piece: SpriteRender,
    pub sprite_horizontal_bar: SpriteRender,
    pub sprite_vertical_bar: SpriteRender,
    pub sprite_cross: SpriteRender,
    pub sprite_queen: SpriteRender,
    pub sprite_protect: SpriteRender,
    pub sprite_sniper: SpriteRender
}

#[derive(Clone)]
pub struct UiElements {
    pub current_team_text: Entity,
    pub current_state_text: Entity,
    pub next_button: Entity,
}

pub struct Actions {
    history: Vec<Box<dyn Action + Sync + Send>>,
    current_move: CompoundAction,
    pos: usize,
    queue: VecDeque<Box<dyn Action + Sync + Send>>
}

impl Actions {
    pub fn new() -> Actions {
        Actions {
            history: Vec::new(),
            current_move: CompoundAction::new(),
            pos: 0, // needed for repeated undo
            queue: VecDeque::new()
        }
    }

    pub fn finish_turn(&mut self) {
        self.assert_empty_queue("finishing turn");
        println!("Finished turn. Pos: {}, History: {:?}", self.pos, self.history.len());
        self.pos = 0;
        self.history.clear();

    }

    pub fn run_queue(&mut self, world: &World) {
        // TODO: Is there another way to avoid a double mutable ownership of self?
        let actions: Vec<Box<dyn Action+Sync+Send>> = {
            self.queue.drain(..).collect()
        };

        for a in actions {
            self.run_action(a, world);
        }
    }

    fn run_action(&mut self, action: Box<dyn Action+Sync+Send>, world: &World) {
        action.get_run_now().run_now(world);
        self.current_move.add(action);
    }

    fn assert_empty_queue(&self, reason: &str) {
        if !self.queue.is_empty() {
            panic!("Action queue had unexpected entries while trying to {}.", reason);
        }
    }

    pub fn add_to_queue(&mut self, action: Box<dyn Action + Send + Sync>) {
        self.queue.push_back(action);
    }

    pub fn insert_only(&mut self, action: Box<dyn Action + Send + Sync>) {
        self.assert_empty_queue("insert new action without running it");
        self.current_move.add(action);
    }

    pub fn insert_and_run(&mut self, action: Box<dyn Action + Send + Sync>, world: &World) {
        self.assert_empty_queue("insert and run new action");
        self.run_action(action, world);
    }

    pub fn finalize_player_move(&mut self) {
        self.assert_empty_queue("finalize player move");

        if self.current_move.is_empty() {
            return;
        }

        let mut finalized_move = CompoundAction::new();
        self.current_move.transfer_content_to(&mut finalized_move);
        self.current_move = CompoundAction::new();
        self.history.push(Box::new(finalized_move));
        self.pos = self.history.len();
    }

    pub fn undo(&mut self, world: &World) {
        self.assert_empty_queue("undo");
        println!("Undoing at pos {}", self.pos);
        if self.pos == 0 {
            return;
        }

        self.pos -= 1;

        let to_be_undone = self.history.get(self.pos).unwrap();
        let anti_action = to_be_undone.get_anti_action();

        anti_action.get_run_now().run_now(world);
        self.history.push(anti_action);
    }
}

impl SimpleState for LoadingState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        world.register::<TurnInto>();
        world.register::<BoardPosition>();
        world.register::<Cell>();
        world.register::<Target>();
        world.register::<Piece>();
        world.register::<Tint>();
        world.register::<Effect>();

        // Get the screen dimensions so we can initialize the camera and
        // place our sprites correctly later. We'll clone this since we'll
        // pass the world mutably to the following functions.
        let dimensions = (*world.read_resource::<ScreenDimensions>()).clone();

        // Place the camera
        init_camera(world, &dimensions);

        let mut actions = Actions::new();

        let sprites_board = load_sprites(world, "squares", 4);
        init_board(world, &sprites_board, &mut actions);

        world.insert(actions);

        world
            .create_entity()
            .with(Mouse{})
            .with(Bounded::new(20., 20.))
            .with(Transform::default())
            .build();


        // let (button_id, button) = UiButtonBuilder::<Transform, u32>::new("NEXT")
        //     .with_id(0)
        //     .with_position(100.,100.)
        //     .build_from_world(world);

        world.exec(|mut creator: UiCreator<'_>| {
            creator.create("prefabs/ui.ron", ());
        });

        let s_vec= load_sprites(world, "pieces", 8);

        world.insert(Sprites{
            sprite_piece: s_vec[0].to_owned(),
            sprite_horizontal_bar: s_vec[1].to_owned(),
            sprite_vertical_bar: s_vec[2].to_owned(),
            sprite_cross: s_vec[3].to_owned(),
            sprite_queen: s_vec[4].to_owned(),
            sprite_protect: s_vec[5].to_owned(),
            sprite_sniper: s_vec[6].to_owned(),
        });


    }

    fn handle_event(
        &mut self,
        mut _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = &event {
            // Check if the window should be closed
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                return Trans::Quit;
            }
        }
        // Keep going
        Trans::None

    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans  {
        let mut current_team_text = None;
        let mut current_status_text = None;
        let mut next_button = None;

        data.world.exec(|finder: UiFinder| {
            if let Some(entity) = finder.find("current_team") {
                println!("Found current team element");
                current_team_text = Some(entity);
            }

            if let Some(entity) = finder.find("current_state") {
                println!("Found current state element");
                current_status_text = Some(entity);
            }

            if let Some(entity) = finder.find("next_button") {
                println!("Found next button element");
                next_button = Some(entity);
            }
        });

        if let Some(next_button_unwrapped) = next_button {
            {
                let mut activatables = data.world.write_storage::<Activatable>();
                activatables.insert(next_button_unwrapped, Activatable { event: BoardEvent::Next });
            }

            let ui_elements = UiElements {
                next_button: next_button_unwrapped,
                current_team_text: current_team_text.unwrap(),
                current_state_text: current_status_text.unwrap()
            };

            data.world.insert::<UiElements>(ui_elements);

            Trans::Replace(Box::new(NextTurnState::first()))
        } else {
            Trans::None
        }
    }
}


fn init_board(world: &mut World, sprites: &[SpriteRender], actions: &mut Actions) {

    let cells = (0..BOARD_WIDTH as usize)
        .map(|x| {
            (0..BOARD_HEIGHT as usize)
                .map(|y| {
                    let scale: f32 = 1.2;
                    let cell_size = 64;
                    let shift_x: f32 = cell_size as f32/2. * scale;
                    let shift_y: f32 = cell_size as f32/2. * scale;

                    let x_pos = ((x * cell_size) as f32) * scale + shift_x;
                    let y_pos = ((y * cell_size) as f32) * scale + shift_y;

                    let x_coord = x as u8;
                    let y_coord = y as u8;

                    let mut transform = Transform::default();
                    transform.set_translation_xyz(x_pos, y_pos, -0.1);
                    transform.set_scale(Vector3::new(scale, scale, scale));

                    let bounded = Bounded {
                        bounds: Cuboid::new(Vector3::new((cell_size - 1) as f32 /2. * scale, (cell_size-1) as f32 /2. * scale, 0.0))
                    };

                    let sprite = &sprites[(x + y)%2];

                    world
                        .create_entity()
                        .with(Cell{})
                        .with(sprite.clone())
                        .with(transform)
                        .with(Activatable{ event: BoardEvent::Cell {x: x_coord,y: y_coord}})
                        .with(bounded)
                        .with(BoardPosition::new(x_coord,y_coord))
                        .with(Target::new())
                        .with(Highlight::new())
                        .build()
                })
                .collect()
        })
        .collect();

    let teams = vec![
        Team {
            name: "Unos",
            id: 0,
            color: Srgba::new(1., 0., 0., 1.),
            lost: false,
        },
         Team {
             name: "Duos",
             id: 1,
             color: Srgba::new(0., 1., 0., 1.),
             lost: false,
         },
    ];

    let team_count = teams.len();

    let board = Board::new(cells, teams);

    for team_id in 0..team_count {
        let piece = world.create_entity()
            .with(Piece::new(team_id))
            .with(TurnInto{kind: PieceKind::Simple})
            .build();

        let pos = BoardPosition::new((1 + team_id * 2) as u8, (1 + team_id * 2) as u8);
        actions.add_to_queue(Place::new(piece, pos));
    }

    world.insert(board);
    world.insert(TurnCounter{num_turns: 0});
}

fn init_camera(world: &mut World, dimensions: &ScreenDimensions) {
    // Center the camera in the middle of the screen, and let it cover
    // the entire screen
    let mut transform = Transform::default();
    transform.set_translation_xyz(dimensions.width() * 0.5, dimensions.height() * 0.5, 1.);

    world
        .create_entity()
        .with(Camera::standard_2d(dimensions.width(), dimensions.height()))
        .with(transform)
        .build();
}

fn load_sprites(world: &mut World, name: &str, count: usize) -> Vec<SpriteRender> {
    // Load the texture for our sprites. We'll later need to
    // add a handle to this texture to our `SpriteRender`s, so
    // we need to keep a reference to it.
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            "sprites/".to_owned() + name + ".png",
            ImageFormat::default(),
            (),
            &texture_storage,
        )
    };

    // Load the spritesheet definition file, which contains metadata on our
    // spritesheet texture.
    let sheet_handle = {
        let loader = world.read_resource::<Loader>();
        let sheet_storage = world.read_resource::<AssetStorage<SpriteSheet>>();
        loader.load(
            "sprites/".to_owned() + name + ".ron",
            SpriteSheetFormat(texture_handle),
            (),
            &sheet_storage,
        )
    };

    // Create our sprite renders. Each will have a handle to the texture
    // that it renders from. The handle is safe to clone, since it just
    // references the asset.
    (0..count)
        .map(|i| SpriteRender {
            sprite_sheet: sheet_handle.clone(),
            sprite_number: i,
        })
        .collect()
}
