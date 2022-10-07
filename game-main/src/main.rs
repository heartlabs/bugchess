mod constants;
mod matchbox;
mod states;

use crate::{
    constants::*,
    states::{core_game_state::CoreGameState, loading::LoadingState, GameState}
};
use game_render::{BoardRender};
use macroquad::prelude::*;
use macroquad_canvas::Canvas2D;

use egui_macroquad::egui;

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
