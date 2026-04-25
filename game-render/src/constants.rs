// ── Canvas dimensions ────────────────────────────────────────────────────

/// Logical canvas width in portrait orientation.
pub const PORTRAIT_CANVAS_W: f32 = 470.0 * 3.;
/// Logical canvas height in portrait orientation.
pub const PORTRAIT_CANVAS_H: f32 = 930.0 * 3.;

/// Logical canvas width in landscape orientation (swapped).
pub const LANDSCAPE_CANVAS_W: f32 = PORTRAIT_CANVAS_H;
/// Logical canvas height in landscape orientation (swapped).
pub const LANDSCAPE_CANVAS_H: f32 = PORTRAIT_CANVAS_W;

// ── Layout-derived constants (identical in portrait & landscape) ──────────

/// The smaller portrait dimension — the board fills the shorter side.
const PORTRAIT_MIN_DIM: f32 = if PORTRAIT_CANVAS_W < PORTRAIT_CANVAS_H {
    PORTRAIT_CANVAS_W
} else {
    PORTRAIT_CANVAS_H
};

/// Board cell size derived from canvas dimensions (smaller dimension ÷ 8).
pub const CELL_WIDTH: f32 = PORTRAIT_MIN_DIM / 8.0;

/// Piece sprite diameter.
pub const PIECE_SCALE: f32 = CELL_WIDTH * 1.2;

/// Font size for UI text (scaled from original 50 @ CELL_WIDTH=135).
pub const FONT_SIZE: f32 = 50.0 * (CELL_WIDTH / REF_CELL_WIDTH);

/// Height of one spare-piece row (scaled from original 96 @ CELL_WIDTH=135).
pub const ROW_HEIGHT: f32 = 96.0 * (CELL_WIDTH / REF_CELL_WIDTH);

/// Reference cell-width of the original 1080-wide canvas (used to scale absolute constants).
pub const REF_CELL_WIDTH: f32 = 135.0;

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
