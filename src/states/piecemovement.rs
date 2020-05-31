use amethyst::{
    assets::{AssetStorage, Loader},
    core::transform::Transform,
    ecs::Entity,
    input::{get_key, get_mouse_button, is_close_requested, is_key_down, VirtualKeyCode},
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture,
               resources::Tint,
                palette::Srgba},
    window::ScreenDimensions,
    shrev::{EventChannel, ReaderId, EventIterator},
    ui::UiEvent,
};

use amethyst::core::math::{Vector3};


use crate::components::{Activatable, Bounded, Mouse, Board, Cell, Piece};

use log::info;
use crate::components::board::{BoardEvent};

use crate::states::PiecePlacementState;


pub struct PieceMovementState {
    pub from_x: usize,
    pub from_y: usize,
    pub piece: Entity,
}

impl SimpleState for PieceMovementState {
    // On start will run when this state is initialized. For more
    // state lifecycle hooks, see:
    // https://book.amethyst.rs/stable/concepts/state.html#life-cycle
    fn on_start(&mut self, _data: StateData<'_, GameData<'_, '_>>) {

    }

    fn handle_event(
        &mut self,
        mut _data: StateData<'_, GameData<'_, '_>>,
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
            StateEvent::Ui(ui_event) => {
                info!(
                    "[HANDLE_EVENT] You just interacted with a ui element: {:?}",
                    ui_event
                );
                Trans::None
            }
            StateEvent::Input(_input) => {
                Trans::None
            }
        }


    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans  {
        let mut board = data.world.write_resource::<Board>();

        if let Some(event) = board.poll_event()
        {
            match event {
                BoardEvent::Cell { x, y } => {
                    println!("Cell Event {},{}", x, y);
                    if let Some(new_piece) = board.get_piece(x, y) {
                        Trans::Replace(Box::new(PieceMovementState { from_x: x, from_y: y, piece: new_piece}))
                    } else {
                        let mut transforms = data.world.write_storage::<Transform>();

                        let transform = &mut transforms.get(board.get_cell(x, y)).unwrap().clone();
                        transform.set_scale(Vector3::new(0.5, 0.5, 1.));

                        transforms.get_mut(self.piece).unwrap().set_translation_xyz(
                            transform.translation().x,
                            transform.translation().y,
                            transform.translation().z);

                        board.move_piece(self.piece, self.from_x, self.from_y, x, y);
                        Trans::Replace(Box::new(PiecePlacementState::new()))
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

impl PieceMovementState {
    fn move_piece(&mut self, world: &mut World, board: &mut Board, x: usize, y: usize) -> SimpleTrans {
        let mut transforms = world.write_storage::<Transform>();

        let transform = &mut transforms.get(board.get_cell(x, y)).unwrap().clone();
        transform.set_scale(Vector3::new(0.5, 0.5, 1.));

        transforms.get_mut(self.piece).unwrap().set_translation_xyz(
            transform.translation().x,
            transform.translation().y,
            transform.translation().z);

        board.move_piece(self.piece, self.from_x, self.from_y, x, y);
        Trans::Replace(Box::new(PiecePlacementState::new()))
    }
}
