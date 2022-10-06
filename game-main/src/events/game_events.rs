use nanoserde::{DeBin, SerBin};

use crate::{rand::rand};

use crate::events::{
    actions::compound_events::{GameAction},
};

use std::{fmt::Debug};

#[derive(Debug, Clone, SerBin, DeBin)]
pub struct GameEventObject {
    pub id: String,
    pub sender: String,
    pub event: GameAction,
}

impl GameEventObject {
    pub const OPCODE: i32 = 1;

    pub fn new(event: GameAction, sender: &String) -> Self {
        GameEventObject {
            id: rand().to_string(),
            sender: sender.clone(),
            event,
        }
    }
}

pub trait EventConsumer {
    fn handle_event(&mut self, event: &GameEventObject);
}
