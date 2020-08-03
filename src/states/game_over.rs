use amethyst::{
    prelude::*,
    ui::{UiText},

};

use crate::{
    components::{

        board::{Team},
    },
    states::{
        load::{UiElements},
    },
    resources::board::{Board,},
};

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

    fn update(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans  {
        Trans::None
    }
}
