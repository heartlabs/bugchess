mod board;
mod conf;
mod constants;
mod game_events;
mod nakama;
mod piece;
mod ranges;
mod rendering;
mod states;

use crate::{
    board::*,
    conf::*,
    constants::*,
    game_events::{BoardEventConsumer, CompoundEventType, EventBroker, EventConsumer, GameEvent},
    nakama::NakamaEventConsumer,
    piece::*,
    ranges::*,
    rendering::{BoardRender, CustomRenderContext},
    states::core_game_state::{CoreGameSubstate},
};

use macroquad::{prelude::*, rand::srand};

use std::{
    borrow::{BorrowMut},
    cell::RefCell,
    rc::Rc,
};
use crate::nakama::NakamaClient;
use crate::states::{
    GameState,
    core_game_state::CoreGameState,
};
use crate::states::loading::LoadingState;

//use wasm_bindgen::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Makrochess".to_owned(),
        window_width: 800,
        window_height: 800,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state: Box<dyn GameState> = Box::new(LoadingState::new());

    loop {
        if let Some(new_state) = state.update() {
            state = new_state;
        }

        state.render();

        next_frame().await;
    }
}




