// ── Canvas dimensions ────────────────────────────────────────────────────

/// Logical canvas width in portrait orientation.
pub const PORTRAIT_CANVAS_W: f32 = 430.0 * 3.;
/// Logical canvas height in portrait orientation.
pub const PORTRAIT_CANVAS_H: f32 = 840.0 * 3.;

/// Logical canvas width in landscape orientation (swapped).
pub const LANDSCAPE_CANVAS_W: f32 = 932.0 * 3.;
/// Logical canvas height in landscape orientation (swapped).
pub const LANDSCAPE_CANVAS_H: f32 = 387.0 * 3.;

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

/// Font size for UI text
pub const FONT_SIZE: f32 = 0.53 * CELL_WIDTH;

/// Height of one spare-piece row
pub const ROW_HEIGHT: f32 = 0.7 * CELL_WIDTH;

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

// ── Pattern infographic ──────────────────────────────────────────────────

/// Size of one cell in the pattern mini-grid.
pub const PATTERN_CELL_SIZE: f32 = 0.28 * CELL_WIDTH;
/// Gap between elements within a pattern card (grid → sprite).
/// Smaller = tighter visual grouping of each pattern with its result piece.
pub const PATTERN_ELEMENT_GAP: f32 = 0.04 * CELL_WIDTH;
/// Gap between rows of pattern cards.
/// Larger = clearer separation between different pattern rows.
pub const PATTERN_ROW_GAP: f32 = 0.80 * CELL_WIDTH;
/// Gap between columns of pattern cards.
/// Larger = clearer separation between different pattern columns.
pub const PATTERN_COL_GAP: f32 = 1.30 * CELL_WIDTH;
/// Size of the resulting piece sprite in the infographic.
pub const PATTERN_PIECE_SIZE: f32 = 0.55 * CELL_WIDTH;
