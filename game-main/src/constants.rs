use game_logic::board::Point2;
use macroquad_canvas::Canvas2D;

#[cfg(target_family = "wasm")]
pub const ONLINE: bool = true;
#[cfg(not(target_family = "wasm"))]
pub const ONLINE: bool = false;

pub const CELL_SCALE: f32 = 1.1875;
pub const CELL_WIDTH: u32 = 64;
pub const CELL_ABSOLUTE_WIDTH: f32 = CELL_WIDTH as f32 * CELL_SCALE;

pub const PIECE_SCALE: f32 = 60.;
pub const SPRITE_WIDTH: f32 = 64.;

pub const SHIFT_X: f32 = 60.;
pub const SHIFT_Y: f32 = 0.;

pub const BOARD_WIDTH: u8 = 8;
pub const BOARD_HEIGHT: u8 = 8;

pub const WINDOW_WIDTH: i32 = 900;
pub const WINDOW_HEIGHT: i32 = 800;

pub const ANIMATION_SPEED: u64 = 400;
pub const PLACE_PIECE_SPEED: u64 = ANIMATION_SPEED;
pub const MOVE_PIECE_SPEED: u64 = ANIMATION_SPEED;
pub const BULLET_SPEED: u64 = ANIMATION_SPEED;
pub const ADD_UNUSED_SPEED: u64 = ANIMATION_SPEED / 3;

pub fn cell_coords(point: &Point2) -> (f32, f32) {
    cell_coords_tuple(point.x, point.y)
}
pub fn cell_coords_tuple(x: u8, y: u8) -> (f32, f32) {
    let x_pos = ((x as u32 * CELL_WIDTH) as f32) * CELL_SCALE + SHIFT_X;
    let y_pos = ((y as u32 * CELL_WIDTH) as f32) * CELL_SCALE + SHIFT_Y;

    (x_pos, y_pos)
}

pub fn coords_to_cell(x_pos: f32, y_pos: f32) -> Point2 {
    let x = (x_pos - SHIFT_X) / CELL_ABSOLUTE_WIDTH;
    let y = (y_pos - SHIFT_Y) / CELL_ABSOLUTE_WIDTH;

    (x as u8, y as u8).into()
}

pub fn cell_hovered(canvas: &Canvas2D) -> Point2 {
    let (mouse_x, mouse_y) = canvas.mouse_position();
    coords_to_cell(mouse_x, mouse_y)
}
