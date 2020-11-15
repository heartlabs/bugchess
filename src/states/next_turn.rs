use amethyst::{
    core::{
        transform::Transform,
        math::{Vector3},
    },
    prelude::*,
    ui::{UiText},
    renderer::{SpriteRender, resources::Tint},
    ecs::{WriteStorage, ReadStorage, WriteExpect, Entities, Join, System, RunNow}
};

use crate::{
    components::{
        board::{BoardPosition,},
        piece::{Piece, }
    },
    states::{
        load::{Sprites,UiElements,},
        PiecePlacementState,
        game_over::GameOverState,
    }
};
use crate::resources::board::{Board,};
use crate::systems::actions::moving::Move;
use crate::systems::actions::place::Place;
use crate::systems::actions::next_turn::*;
use crate::states::load::Actions;

use std::io::prelude::*;
use std::net::{TcpStream, TcpListener};
use std::time::Duration;

pub struct TurnCounter {
    pub num_turns: u32,
}

pub struct NextTurnState {
    pub(crate) first_turn: bool,
    pub(crate) no_teams_left: bool,
}

impl NextTurnState {
    pub fn new() -> NextTurnState {
        NextTurnState {
            first_turn: false,
            no_teams_left: false
        }
    }

    pub fn first() -> NextTurnState {
        NextTurnState {
            first_turn: true,
            no_teams_left: false
        }
    }
}

impl SimpleState for NextTurnState {

    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        {
            let mut turn_counter = data.world.write_resource::<TurnCounter>();
            turn_counter.num_turns += 1;
        }
        let mut systems: Vec<Box<dyn RunNow>> = Vec::new();
        if !self.first_turn {
            systems.push(Box::new(IdentifyLosingTeams {}));
        }
        systems.push(Box::new(NextTeam { state: self }));

        for mut s in systems {
            s.run_now(&data.world);
        }
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans  {
        {
            let board = data.world.write_resource::<Board>();
            
            {
                let mut ui_text = data.world.write_storage::<UiText>();
                let team = board.current_team();
                let ui_elements = data.world.read_resource::<UiElements>();
                if let Some(text) = ui_text.get_mut(ui_elements.current_team_text) {
                    text.text = format!("Current Team: {}", team.name);
                    text.color = [team.color.red, team.color.green, team.color.blue, team.color.alpha];
                }
            }

            if self.no_teams_left || board.num_unused_pieces() > 20 {
                return Trans::Replace(Box::new(GameOverState::new(board.current_team())))
            }
        }

        let mut actions = data.world.write_resource::<Actions>();
        actions.run_queue(data.world);
        actions.finalize_player_move();
        actions.finish_turn(data.world);

        Trans::Replace(Box::new(PiecePlacementState::new()))
    }
}
