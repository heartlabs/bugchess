pub mod core_game_state;
pub mod loading;

use macroquad_canvas::Canvas2D;

pub trait GameState {
    fn update(&mut self, canvas: &Canvas2D) -> Option<Box<dyn GameState>>;
    fn render(&self, canvas: &Canvas2D);
    fn uses_egui(&self) -> bool;
    /// Called when the canvas has been resized (e.g. orientation change).
    /// Default implementation is a no-op.
    fn handle_resize(&mut self, _new_w: f32, _new_h: f32) {}
}
