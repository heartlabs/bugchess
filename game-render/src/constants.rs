// ── Layout-derived constants (identical in portrait & landscape) ──────────

/// Board cell size derived from the standard canvas width (1080 ÷ 8 = 135).
pub const CELL_WIDTH: f32 = 135.0;

/// Piece sprite diameter (≈84 at CELL_WIDTH=135).
pub const PIECE_SCALE: f32 = CELL_WIDTH * 1.2;

/// Font size for UI text (≈46 at CELL_WIDTH=135).
pub const FONT_SIZE: f32 = 50.0;

/// Height of one spare-piece row (≈96). Pieces may overlap slightly vertically.
pub const ROW_HEIGHT: f32 = 96.0;

/// Line-spacing multiplier for multi-line description text.
pub const TEXT_LINE_SPACING: f32 = 1.3;

// ── Timing constants ───────────────────────────────────────────────────────

pub const ANIMATION_SPEED: u64 = 400;
pub const PLACE_PIECE_SPEED: u64 = ANIMATION_SPEED;
pub const MOVE_PIECE_SPEED: u64 = ANIMATION_SPEED;
pub const BULLET_SPEED: u64 = ANIMATION_SPEED;
pub const ADD_UNUSED_SPEED: u64 = ANIMATION_SPEED / 3;

// ── Board geometry ─────────────────────────────────────────────────────────

pub const BOARD_WIDTH: u8 = 8;
pub const BOARD_HEIGHT: u8 = 8;
