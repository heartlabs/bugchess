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
use crate::systems::actions::actions::{Action, HasRunNow};

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
    pub(crate) on_start: Vec<Box<dyn HasRunNow + Sync + Send>>,
}

impl Actions {
    pub fn new() -> Actions {
        Actions {
            on_start: vec![
                Box::new(UpdateUi{text: "Place your Piece"}),
                Box::new(InitNewPieces{}),
                Box::new(MergePiecePatterns{}),
                Box::new(InitNewPieces{}),
                Box::new(UpdateTargets{}),
            ]
        }
    }
}

pub struct Ax {

}

impl Ax {
    pub fn get_actions<'a>(&self) -> Vec<Box<dyn RunNow<'a> + 'a>>{
        vec![
            Box::new(UpdateUi{text: "Place your Piece"}),
            Box::new(InitNewPieces{}),
            Box::new(MergePiecePatterns{}),
            Box::new(InitNewPieces{}),
            Box::new(UpdateTargets{}),
        ]
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

        let sprites_board = load_sprites(world, "squares", 4);
        init_board(world, &sprites_board);

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

        world.insert(Actions::new());
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


fn init_board(world: &mut World, sprites: &[SpriteRender]) {

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
        world.create_entity()
            .with(Piece::new(team_id))
            .with(BoardPosition::new((1 + team_id * 2) as u8, (1 + team_id * 2) as u8))
            .with(TurnInto{kind: PieceKind::Simple})
            .build();
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
