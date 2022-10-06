mod constants;
mod events;
//mod game_logic;
mod matchbox;
mod rendering;
mod states;

use crate::{
    constants::*,
    rendering::{BoardRender, CustomRenderContext},
};
use game_logic::{board::*, piece::*, ranges::*};

use macroquad::prelude::*;
use macroquad_canvas::Canvas2D;

use crate::states::{core_game_state::CoreGameState, loading::LoadingState, GameState};
use egui_macroquad::egui;
use events::{
    actions::compound_events::GameAction, atomic_events::AtomicEvent,
    board_event_consumer::BoardEventConsumer,
};
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

        if state.uses_egui() {
            egui_macroquad::draw();
        }

        next_frame().await;
    }
}
