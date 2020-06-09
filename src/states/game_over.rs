use amethyst::{
    core::{
        transform::Transform,
        math::{Vector3, Point2},
    },
    input::{is_close_requested, is_key_down, VirtualKeyCode},
    prelude::*,
    renderer::{SpriteRender, resources::Tint},
    ui::{UiText, UiFinder, UiEventType, UiEvent},
    ecs::{WriteStorage, ReadStorage, ReadExpect, WriteExpect, Entity, Entities, Join},
};

use crate::{
    components::{
        Activatable, Piece,
        board::{BoardEvent, Team},
    },
    states::{
        load::Sprites,
        PieceMovementState,
    }
};
use crate::states::load::UiElements;
use crate::components::board::{Move, Range, Direction, BoardPosition, Target, TurnInto, Dying, PieceKind};
use crate::components::{Cell, Bounded};
use crate::resources::board::{Board, Pattern, PatternComponent};
use crate::components::board::PieceKind::{HorizontalBar, Simple};
use crate::states::PiecePlacementState;


pub struct GameOverState {
    winning_team: Team,
}

impl GameOverState {
    pub fn new(winning_team: Team) -> GameOverState {
        GameOverState {
            winning_team
        }
    }

}

impl SimpleState for GameOverState {

    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {

        {
            let mut ui_text = data.world.write_storage::<UiText>();
            let ui_elements = data.world.read_resource::<UiElements>();
            let board = data.world.read_resource::<Board>();

            if let Some(text) = ui_text.get_mut(ui_elements.current_state_text) {
                text.text = format!("Team {} won!", board.current_team().name).parse().unwrap();
            }
        }
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans  {
        Trans::None
    }
}
