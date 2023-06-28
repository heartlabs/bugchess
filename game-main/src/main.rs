mod constants;
mod matchbox;
//mod nakama;
//mod custom_client;
mod states;

use crate::{
    constants::*,
    states::{loading::LoadingState, GameState},
};

use macroquad::{prelude::*, rand::srand};
use macroquad_canvas::Canvas2D;

#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

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
    let canvas = Canvas2D::new(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32);
    let mut state = setup_game_state().await;

    loop {
        srand((macroquad::time::get_time() * 100_000_000f64) as u64);
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

#[cfg(target_family = "wasm")]
fn get_url_param(key: &str) -> Option<String> {
    web_sys::window()
        .as_ref()
        .and_then(web_sys::Window::document)
        .and_then(|document| document.url().ok())
        .and_then(|url| url::Url::parse(&url).ok())
        .and_then(|url| {
            url.query_pairs()
                .filter(|q| q.0 == key)
                .map(|(_k, v)| (*v).to_string())
                .next()
        })
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
extern "C" {
    fn getProperty(name: &str) -> wasm_bindgen::JsValue;
}

async fn setup_game_state() -> Box<dyn GameState> {
    #[cfg(target_family = "wasm")]
    let preconfigured_room_id = getProperty("room_id").as_string();
    #[cfg(not(target_family = "wasm"))]
    let preconfigured_room_id: Option<String> = None;

    next_frame().await;
    next_frame().await;

    let seed_time = macroquad::time::get_time();
    let seed = (seed_time * 100_000_000f64) as u64;
    info!("Initial seed from {} is {}", seed_time, seed);
    srand(seed);

    let mut loading_state = LoadingState::new();
    if let Some(room_id) = preconfigured_room_id.as_ref() {
        if !room_id.is_empty() {
            info!("Was preconfigured with room_id {}", room_id);
            loading_state.join_room(room_id.as_str());
        }
    }

    #[cfg(target_family = "wasm")]
    if let Some(true) = getProperty("offline").as_bool() {
        info!("Offline game");
        loading_state.offline_game();
    }

    Box::new(loading_state)
}
