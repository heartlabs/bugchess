use nanoserde::{DeBin, SerBin};

use crate::{
    info,
    rand::rand,
};

use macroquad::logging::warn;
use std::{
    cell::RefCell,
    rc::Rc,
};
use std::fmt::Debug;
use std::slice::Iter;
use crate::events::atomic_events::AtomicEvent;
use crate::events::atomic_events::AtomicEvent::*;
use crate::events::compound_events::{CompoundEvent, GameAction};

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
