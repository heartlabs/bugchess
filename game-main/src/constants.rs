use macroquad::prelude::{screen_height, screen_width};

/// Fixed logical canvas width for portrait orientation (1080 = CELL_WIDTH × 8).
pub const PORTRAIT_CANVAS_W: f32 = 1080.0;
/// Fixed logical canvas height for portrait orientation.
pub const PORTRAIT_CANVAS_H: f32 = 1800.0;

/// Fixed logical canvas width for landscape orientation.
pub const LANDSCAPE_CANVAS_W: f32 = 1920.0;
/// Fixed logical canvas height for landscape orientation (1080 = CELL_WIDTH × 8).
pub const LANDSCAPE_CANVAS_H: f32 = 1080.0;

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
