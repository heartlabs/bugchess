
#[cfg(target_family = "wasm")]
pub const ONLINE: bool = true;
#[cfg(not(target_family = "wasm"))]
pub const ONLINE: bool = false;


pub const WINDOW_WIDTH: i32 = 900;
pub const WINDOW_HEIGHT: i32 = 800;




