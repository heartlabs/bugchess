use macroquad::prelude::{screen_height, screen_width};

// Canvas dimensions live in game-render so CELL_WIDTH can derive from them.
pub use game_render::constants::{
    LANDSCAPE_CANVAS_H, LANDSCAPE_CANVAS_W, PORTRAIT_CANVAS_H, PORTRAIT_CANVAS_W,
};

/// Default window width (used for Conf; portrait as the common orientation).
pub const WINDOW_WIDTH: i32 = PORTRAIT_CANVAS_W as i32;
/// Default window height (used for Conf).
pub const WINDOW_HEIGHT: i32 = PORTRAIT_CANVAS_H as i32;

/// Return the fixed logical canvas dimensions for the current orientation.
/// Call only after the renderer is initialised (`screen_width`/`screen_height` available).
pub fn logical_canvas_size() -> (f32, f32) {
    if screen_height() > screen_width() {
        (PORTRAIT_CANVAS_W, PORTRAIT_CANVAS_H)
    } else {
        (LANDSCAPE_CANVAS_W, LANDSCAPE_CANVAS_H)
    }
}
