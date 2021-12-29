use crate::*;
use macroquad::prelude::mouse_position;

pub const CELL_SCALE: f32 = 1.1875;
pub const CELL_WIDTH: u32 = 64;
pub const CELL_ABSOLUTE_WIDTH: f32 = CELL_WIDTH as f32 * CELL_SCALE;

pub const SHIFT_X: f32 = CELL_WIDTH as f32 / 2. * CELL_SCALE;
pub const SHIFT_Y: f32 = CELL_WIDTH as f32 / 2. * CELL_SCALE;

pub const BOARD_WIDTH: u8 = 8;
pub const BOARD_HEIGHT: u8 = 8;

pub fn cell_coords(x: u8, y: u8) -> (f32, f32) {
    let x_pos = ((x as u32 * CELL_WIDTH) as f32) * CELL_SCALE + SHIFT_X;
    let y_pos = ((y as u32 * CELL_WIDTH) as f32) * CELL_SCALE + SHIFT_Y;

    (x_pos, y_pos)
}

pub fn coords_to_cell(x_pos: f32, y_pos: f32) -> (u8, u8) {
    let x = (x_pos - SHIFT_X) / CELL_ABSOLUTE_WIDTH;
    let y = (y_pos - SHIFT_Y) / CELL_ABSOLUTE_WIDTH;

    (x as u8, y as u8)
}

pub fn cell_hovered() -> (u8, u8) {
    let (mouse_x, mouse_y) = mouse_position();
    let (mouse_cell_x, mouse_cell_y) = coords_to_cell(mouse_x, mouse_y);
    (mouse_cell_x, mouse_cell_y)
}
