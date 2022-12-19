use nanoserde::{DeBin, SerBin};

use quad_rand::rand;

use crate::actions::compound_events::GameAction;

use std::fmt::Debug;

#[derive(Debug, Clone, SerBin, DeBin)]
pub enum Event {
    GameAction(GameAction),
    PlayerAction(PlayerAction),
}

#[derive(Debug, Clone, SerBin, DeBin)]
pub enum PlayerAction {
    Connect(String), // player name
    SendGame(Vec<GameAction>)
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
