use amethyst::{
    assets::{AssetStorage, Loader},
    core::transform::Transform,
    input::{get_key, get_mouse_button, is_close_requested, is_key_down, VirtualKeyCode},
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture,
               resources::Tint,
                palette::Srgba},
    window::ScreenDimensions,
    shrev::{EventChannel, ReaderId, EventIterator},
    ui::{RenderUi, UiBundle, UiCreator, UiEvent, UiFinder, UiText},
};

use amethyst::core::math::{Vector3, Point2};
use ncollide3d::shape::Cuboid;

use crate::components::{Activatable, Bounded, Mouse, Board, Cell, Piece};

use log::info;
use crate::components::board::{BoardEvent, Team};
use crate::states::PiecePlacementState;
use std::any::Any;

pub struct LoadingState;

#[derive(Clone)]
pub struct Sprites {
    pub sprite_piece: SpriteRender,
}

impl SimpleState for LoadingState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

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


        world
            .create_entity()
            .with(sprites_board[3].clone())
            .with(Bounded::new(64., 64.))
            .with(Transform::default())
            .with(Activatable{active: false, event: BoardEvent::Next})
            .build();

        // let (button_id, button) = UiButtonBuilder::<Transform, u32>::new("NEXT")
        //     .with_id(0)
        //     .with_position(100.,100.)
        //     .build_from_world(world);

        world.exec(|mut creator: UiCreator<'_>| {
            creator.create("prefabs/ui.ron", ());
        });

        let s_vec= load_sprites(world, "piece", 1);
        let sprite_piece: SpriteRender = s_vec.first().unwrap().clone();

        world.insert(Sprites{sprite_piece });

        world.register::<Team>();
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
        Trans::Replace(Box::new(PiecePlacementState::new()))
    }

    fn update(&mut self, mut data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans  {
        Trans::None
    }
}


fn init_board(world: &mut World, sprites: &[SpriteRender]) {

    let cells = (0..5)
        .map(|x| {
            (0..5)
                .map(|y| {
                    let cell = Cell {
                        coords: Point2::<usize>::new(x,y),
                        piece: None
                    };

                    let scale: f32 = 2.;
                    let shift_x: f32 = 100. * scale;
                    let shift_y: f32 = 64. * scale;

                    let x_pos = ((x * Cell::width) as f32) * scale + shift_x;
                    let y_pos = ((y * Cell::height) as f32) * scale + shift_y;

                    let mut transform = Transform::default();
                    transform.set_translation_xyz(x_pos, y_pos, 0.);
                    transform.set_scale(Vector3::new(scale, scale, scale));

                    let bounded = Bounded {
                        bounds: Cuboid::new(Vector3::new((Cell::width - 1) as f32 /2. * scale, (Cell::height-1) as f32 /2. * scale, 0.0))
                    };

                    let sprite = &sprites[(x + y)%2];

                    world
                        .create_entity()
                        .with(cell)
                        .with(sprite.clone())
                        .with(transform)
                        .with(Activatable{active: false, event: BoardEvent::Cell {x,y}})
                        .with(bounded)
                        .build()
                })
                .collect()
        })
        .collect();

    let teams = vec![
        Team {
            name: "Unos",
            id: 1,
            color: Srgba::new(1., 0., 0., 1.)
        },
         Team {
             name: "Duos",
             id: 2,
             color: Srgba::new(0., 1., 0., 1.)
         },
    ];
    let board = Board::new(cells, teams);

    world.insert(board);
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
