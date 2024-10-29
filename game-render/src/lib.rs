mod animation;
pub mod constants;
pub mod render_events;
pub mod rendering;
pub mod sprite;
pub mod ui;

pub use rendering::*;

pub fn draw_ui() {
    egui_macroquad::draw();
}
