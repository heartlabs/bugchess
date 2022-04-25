mod board;
mod constants;
mod game_events;
mod matchbox;
mod piece;
mod ranges;
mod rendering;
mod states;
mod ui;

use crate::{
    board::*,
    constants::*,
    game_events::{BoardEventConsumer, CompoundEventType, EventBroker, GameEvent},
    piece::*,
    ranges::*,
    rendering::{BoardRender, CustomRenderContext},
};

use macroquad::{prelude::*, rand::srand};
use macroquad_canvas::Canvas2D;

use crate::states::loading::LoadingState;
use crate::states::{core_game_state::CoreGameState, GameState};
use std::{borrow::BorrowMut, cell::RefCell, rc::Rc};

//use wasm_bindgen::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Makrochess".to_owned(),
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state: Box<dyn GameState> = Box::new(LoadingState::new());
    let canvas = Canvas2D::new(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32);

    loop {
        set_camera(&canvas.camera);
        clear_background(BLACK);

        if let Some(new_state) = state.update(&canvas) {
            state = new_state;
        }
        state.render(&canvas);

        set_default_camera();

        clear_background(BLACK);
        canvas.draw();

        next_frame().await;
    }
}
