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
    ui::{UiText, UiFinder, UiEventType, UiEvent},
    ecs::Entity,
};

use amethyst::core::math::{Vector3};


use crate::components::{Activatable, Bounded, Mouse, Board, Cell, Piece};


use crate::components::board::{BoardEvent, Team};
use crate::states::load::Sprites;
use crate::states::PieceMovementState;


pub struct PiecePlacementState {
    current_team_text: Option<Entity>,
    next_button: Option<Entity>,
}

impl PiecePlacementState {
    pub fn new() -> PiecePlacementState {
        PiecePlacementState {
            current_team_text: None,
            next_button: None,
        }
    }
}

impl SimpleState for PiecePlacementState {

    fn on_start(&mut self, _data: StateData<'_, GameData<'_, '_>>) {

    }

    fn handle_event(
        &mut self,
        _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        match &event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                    Trans::Quit
                } else {
                    Trans::None
                }
            }
            StateEvent::Ui(UiEvent{target: _, event_type: UiEventType::ClickStart}) => {
                Trans::None
            }
            StateEvent::Input(_input) => {
                Trans::None
            }
            _ => Trans::None
        }
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans  {

        if self.current_team_text.is_none() {
            data.world.exec(|finder: UiFinder| {
                if let Some(entity) = finder.find("current_team") {
                    println!("Found current team element");
                    self.current_team_text = Some(entity);
                }

                if let Some(entity) = finder.find("next_button") {
                    println!("Found next button element");
                    self.next_button = Some(entity);
                }
            });

            let mut activatables = data.world.write_storage::<Activatable>();
            if self.next_button.is_some() {
                activatables.insert(self.next_button.unwrap(), Activatable { active: false, event: BoardEvent::Next });
            }
        }

        let mut board = data.world.write_resource::<Board>();

        {
            let mut ui_text = data.world.write_storage::<UiText>();
            let team = board.current_team();
            if let Some(text) = self.current_team_text.and_then(|entity| ui_text.get_mut(entity)) {
                text.text = format!("Current Team: {}", team.name);
                text.color = [team.color.red, team.color.green, team.color.blue, team.color.alpha];
            }
        }

        if let Some(event) = board.poll_event() {
            match event {
                BoardEvent::Cell { x, y } => {
                    println!("Cell Event {},{}", x, y);
                    let mut teams = data.world.write_storage::<Team>();

                    if let Some(piece) = board.get_piece(x, y) {
                        //board.remove_piece(x, y);
                        //data.world.entities().delete(piece);
                        if teams.get(piece).unwrap().id == board.current_team().id {
                            println!("Moving piece");
                            Trans::Replace(Box::new(PieceMovementState { from_x: x, from_y: y, piece }))
                        } else {
                            Trans::None
                        }
                    } else {
                        let mut pieces = data.world.write_storage::<Piece>();
                        let mut transforms = data.world.write_storage::<Transform>();
                        let mut tints = data.world.write_storage::<Tint>();
                        let mut sprite_renders = data.world.write_storage::<SpriteRender>();

                        let sprites = data.world.read_resource::<Sprites>();

                        let piece = Piece {};

                        let cell_entity = board.get_cell(x, y);

                        let transform = &mut transforms.get(cell_entity).unwrap().clone();
                        transform.set_scale(Vector3::new(0.5, 0.5, 1.));

                        let team = board.current_team();
                        let piece_entity = data.world.entities().build_entity()
                            .with(piece.clone(), &mut pieces)
                            .with(transform.clone(), &mut transforms)
                            .with(sprites.sprite_piece.clone(), &mut sprite_renders)
                            .with(Tint(team.color), &mut tints)
                            .with(team, &mut teams)
                            .build();

                        board.placePiece(piece_entity, x, y);

                        println!("Placed new piece");
                        Trans::None
                    }
                },
                BoardEvent::Next => {
                    board.next_team();
                    Trans::Replace(Box::new(PiecePlacementState::new()))
                },
                _ => Trans::None
            }
        } else {
            Trans::None
        }



    }
}

impl PiecePlacementState {
    fn place_piece(&self, world: &mut World, board: &mut Board, x: usize, y: usize) -> SimpleTrans {
        println!("Cell Event {},{}", x, y);

        if let Some(piece) = board.get_piece(x, y) {
            //board.remove_piece(x, y);
            //data.world.entities().delete(piece);
            println!("Moved piece");
            Trans::Replace(Box::new(PieceMovementState { from_x: x, from_y: y, piece}))
        } else {
            let mut pieces = world.write_storage::<Piece>();
            let mut transforms = world.write_storage::<Transform>();
            let mut tints = world.write_storage::<Tint>();
            let mut sprite_renders = world.write_storage::<SpriteRender>();
            let mut teams = world.write_storage::<Team>();

            let sprites = world.read_resource::<Sprites>();

            let piece = Piece {};

            let cell_entity = board.get_cell(x, y);

            let transform = &mut transforms.get(cell_entity).unwrap().clone();
            transform.set_scale(Vector3::new(0.5, 0.5, 1.));

            let team = board.current_team();
            let piece_entity = world.entities().build_entity()
                .with(piece.clone(), &mut pieces)
                .with(transform.clone(), &mut transforms)
                .with(sprites.sprite_piece.clone(), &mut sprite_renders)
                .with(Tint(team.color), &mut tints)
                .with(team, &mut teams)
                .build();

            board.placePiece(piece_entity, x, y);

            println!("Placed new piece");
            Trans::None
        }
    }
}
