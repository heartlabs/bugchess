pub const CELL_SCALE: f32 = 1.1875;
pub const CELL_WIDTH: u32 = 64;

pub fn cell_coords(x:u8, y:u8) -> (f32,f32) {
    let shift_x: f32 = CELL_WIDTH as f32/2. * CELL_SCALE;
    let shift_y: f32 = CELL_WIDTH as f32/2. * CELL_SCALE;

    let x_pos = ((x as u32 * CELL_WIDTH) as f32) * CELL_SCALE + shift_x;
    let y_pos = ((y as u32 * CELL_WIDTH) as f32) * CELL_SCALE + shift_y;

    (x_pos, y_pos)
}