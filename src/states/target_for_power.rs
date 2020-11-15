use amethyst::{
    ui::UiText,
    ecs::{ Entity},
    input::{is_close_requested, is_key_down, VirtualKeyCode},
    prelude::*,
};

use crate::{
    components::{Activatable,
                 active::Selected,
                 board::{BoardEvent, BoardPosition, Target, Team},
                 piece::{Piece, ActivatablePower, Power},
    },

    states::{
        PiecePlacementState,
        load::UiElements,
        next_turn::NextTurnState,
    },
    resources::board::Board,
};

use log::info;
use crate::components::bounded::PowerAnimation;
use amethyst::core::Transform;
use crate::components::Cell;
use amethyst::core::math::Vector3;
use std::time::Instant;
use crate::states::load::Sprites;
use amethyst::renderer::{Sprite, SpriteRender, resources::Tint};
use crate::constants::cell_coords;
use crate::states::piecemovement::POWER_ANIMATION_DURATION;

pub struct TargetForPowerState {
    pub from_x: u8,
    pub from_y: u8,
    pub piece: Entity,
}

impl SimpleState for TargetForPowerState {
    // On start will run when this state is initialized. For more
    // state lifecycle hooks, see:
    // https://book.amethyst.rs/stable/concepts/state.html#life-cycle
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        {
            let mut ui_text = data.world.write_storage::<UiText>();
            let ui_elements = data.world.read_resource::<UiElements>();
            if let Some(text) = ui_text.get_mut(ui_elements.current_state_text) {
                text.text = "Choose a target for your special power.".parse().unwrap();
            }
        }

        let mut selected = data.world.write_storage::<Selected>();
        let board = data.world.read_resource::<Board>();
        let cell = board.get_cell(self.from_x, self.from_y);

        selected.insert(cell, Selected{});

    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let mut selected = data.world.write_storage::<Selected>();
        let board = data.world.read_resource::<Board>();
        let cell = board.get_cell(self.from_x, self.from_y);

        selected.remove(cell);
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
                BoardEvent::Next => {
                    return Trans::Replace(Box::new(NextTurnState::new()));
                },
                BoardEvent::Cell { x, y } => {
                    println!("Cell Event {},{}", x, y);

                    let mut pieces = data.world.write_storage::<Piece>();

                    let mut piece_at_target = board.get_piece(x, y).and_then(|e| pieces.get_mut(e));

                    if let Some(new_piece) = piece_at_target
                    {
                        if new_piece.team_id == board.current_team().id {
                            return Trans::None;
                        }

                        piece_at_target = Some(new_piece);
                    }

                    let mut targets = data.world.write_storage::<Target>();
                    let mut transforms = data.world.write_storage::<Transform>();
                    let mut sprite_renders = data.world.write_storage::<SpriteRender>();
                    let mut tints = data.world.write_storage::<Tint>();
                    let sprites = data.world.read_resource::<Sprites>();

                    let cell = board.get_cell(x,y);
                    let target = targets.get(cell).unwrap();

                    let mut power_animations = data.world.write_storage::<PowerAnimation>();
                    if target.is_possible_special_target_of(self.piece) {
                        if let Some(attacked_piece) = piece_at_target {
                            attacked_piece.dying = true;
                            let mut own_piece = pieces.get_mut(self.piece).unwrap();
                            own_piece.exhaustion.on_attack();

                            let (x_coord, y_coord) = cell_coords(x,y);
                            let target_pos = Vector3::new(x_coord, y_coord, 0.);

                            {
                                let own_transform = transforms.get(self.piece).unwrap();
                                let power_animation: PowerAnimation = PowerAnimation {
                                    from_pos: own_transform.translation().clone(),
                                    to_pos: target_pos,
                                    start_time: Instant::now(),
                                    duration: POWER_ANIMATION_DURATION,
                                    start_scale: 1.0,
                                    end_scale: 1.0
                                };

                                data.world.entities().build_entity()
                                    .with(power_animation, &mut power_animations)
                                    .with(own_transform.clone(), &mut transforms)
                                    .with(sprites.sprite_bullet.clone(), &mut sprite_renders)
                                    .build();
                            }
                            {
                                let own_transform = transforms.get(self.piece).unwrap();
                                let dying_animation: PowerAnimation = PowerAnimation {
                                    from_pos: target_pos,
                                    to_pos: target_pos,
                                    start_time: Instant::now(),
                                    duration: POWER_ANIMATION_DURATION,
                                    start_scale: 1.0,
                                    end_scale: 1.0
                                };

                                let target_entity = board.get_piece(x, y).unwrap();
                                data.world.entities().build_entity()
                                    .with(dying_animation, &mut power_animations)
                                    .with(own_transform.clone(), &mut transforms)
                                    .with(sprite_renders.get(target_entity).unwrap().to_owned(), &mut sprite_renders)
                                    .with(tints.get(target_entity).unwrap().to_owned(), &mut tints)
                                    .build();
                            }
                        }
                    }

                    return Trans::Replace(Box::new(PiecePlacementState::new()));

                },
                _ => { }
            }
        }
        Trans::None
    }
}
