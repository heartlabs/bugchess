//! Rendering layer for Bugchess — draws the board, pieces, and UI using macroquad.
//!
//! [`rendering::CustomRenderContext`] owns textures and visual state; [`sprite`] maps piece kinds
//! to sprite-sheet regions and colours; [`animation`] interpolates piece movements and effects
//! over time; [`ui`] provides simple button widgets for undo and next-turn.
//!
//! Depends on `game-core` and `game-model`; consumed by `game-main`.

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
