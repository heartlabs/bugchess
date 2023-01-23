use nanoserde::{DeJson, SerJson};

use quad_rand::rand;

use game_events::actions::compound_events::GameAction;

use crate::game_controller::GameCommand;
use std::fmt::{Debug, Display};

#[derive(Debug, Clone, SerJson, DeJson)]
pub enum Event {
    GameCommand(GameCommand),
    PlayerAction(PlayerAction),
}

#[derive(Debug, Clone, SerJson, DeJson)]
pub enum PlayerAction {
    /// player name, index
    Connect(String, usize),
    /// client ids of players in order
    NewGame((String, String)),
}

#[derive(Debug, Clone, SerJson, DeJson)]
pub struct GameEventObject {
    pub id: String,
    pub sender: String,
    pub event: Event,
}

impl GameEventObject {
    pub const OPCODE: i32 = 1;

    pub fn new(event: Event, sender: &String) -> Self {
        GameEventObject {
            id: rand().to_string(),
            sender: sender.clone(),
            event,
        }
    }
}

impl Display for GameEventObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.event {
            Event::GameCommand(game_action) => write!(
                f,
                "GameEventObject {} with GameAction {}",
                self.id, game_action
            ),
            Event::PlayerAction(player_action) => write!(
                f,
                "GameEventObject {} with PlayerAction {:?}",
                self.id, player_action
            ),
        }
    }
}
