use game_model::game::Team;
use nanoserde::{DeBin, SerBin};

use quad_rand::rand;

use crate::actions::compound_events::GameAction;

use std::fmt::{Debug, Display};

#[derive(Debug, Clone, SerBin, DeBin)]
pub enum Event {
    GameAction(GameAction),
    PlayerAction(PlayerAction),
}

#[derive(Debug, Clone, SerBin, DeBin)]
pub enum PlayerAction {
    Connect(String, usize), // player name, index
    NewGame((String, String)) // client ids of players in order
}

#[derive(Debug, Clone, SerBin, DeBin)]
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

pub trait EventConsumer {
    fn handle_event(&mut self, event: &GameAction);
}

impl Display for GameEventObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        match &self.event {
            Event::GameAction(game_action) => write!(f, "GameEventObject {} with GameAction {}", self.id, game_action ),
            Event::PlayerAction(player_action) => write!(f, "GameEventObject {} with PlayerAction {:?}", self.id, player_action ),
        }
        
    }
}