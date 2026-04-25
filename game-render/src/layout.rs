use crate::constants::{CELL_WIDTH, PIECE_SCALE, REF_CELL_WIDTH, ROW_HEIGHT};

// ── Orientation-specific layout constants ─────────────────────────────────

/// Scaling factor for constants that were designed at REF_CELL_WIDTH.
/// Applies to all absolute-pixel values below.
const SCALE: f32 = CELL_WIDTH / REF_CELL_WIDTH;

/// Gap between layout regions as a fraction of cell width (portrait) — already relative.
const PORTRAIT_GAP_FACTOR: f32 = 0.25;
/// Height of interactive buttons in portrait mode (scaled from 70 @ REF_CELL_WIDTH=135).
const PORTRAIT_BTN_HEIGHT: f32 = 70.0 * SCALE;
/// Width of the "End Turn" button in portrait mode (scaled from 350).
const PORTRAIT_BTN0_WIDTH: f32 = 350.0 * SCALE;
/// Width of the "Undo" button in portrait mode (scaled from 300).
const PORTRAIT_BTN1_WIDTH: f32 = 300.0 * SCALE;
/// Gap between the two side-by-side buttons in portrait mode (scaled from 30).
const PORTRAIT_BTN_GAP: f32 = 30.0 * SCALE;
/// Number of spare-piece columns in portrait (pieces spread across entire width)
const PORTRAIT_SPARE_COLS: u32 = 20;

/// Top of the button area in landscape mode (scaled from 18).
const LANDSCAPE_BTN_TOP: f32 = 18.0 * SCALE;
/// Height of each stacked button in landscape mode (scaled from 84).
const LANDSCAPE_BTN_HEIGHT: f32 = 84.0 * SCALE;
/// Horizontal padding from the left column edge in landscape mode (scaled from 10).
const LANDSCAPE_BTN_PAD: f32 = 10.0 * SCALE;
/// Gap between the two stacked buttons in landscape mode (scaled from 10).
const LANDSCAPE_BTN_GAP: f32 = 10.0 * SCALE;
/// Gap between buttons and the first spare row in landscape mode (scaled from 24).
const LANDSCAPE_SPARE0_GAP: f32 = 24.0 * SCALE;
/// Gap between spare row groups in landscape mode (scaled from 32).
const LANDSCAPE_SPARE_GAP: f32 = 32.0 * SCALE;
/// Gap before the text area in landscape mode (scaled from 50).
const LANDSCAPE_TEXT_GAP: f32 = 50.0 * SCALE;
/// Number of spare-piece columns per row in landscape (fits in left column)
const LANDSCAPE_SPARE_COLS: u32 = 7;


use game_model::Point2;
use macroquad::math::Rect;
use macroquad_canvas::Canvas2D;

/// Immutable layout snapshot, recomputed whenever the canvas resizes.
/// Create via [`compute_layout`], pass by `&` reference everywhere.
///
/// Layout-independent constants (`CELL_WIDTH`, `PIECE_SCALE`, `FONT_SIZE`,
/// `ROW_HEIGHT`) live in [`crate::constants`] and are shared by both
/// portrait and landscape.
#[derive(Debug, Clone, Copy)]
pub struct LayoutConstants {
    /// Board x offset (0 portrait, left_col landscape)
    pub shift_x: f32,
    /// Board y offset (always 0)
    pub shift_y: f32,
    /// X position of first description text line (board-left in portrait, left-col in landscape)
    pub text_x: f32,
    /// Y position of first description text line
    pub text_y: f32,
    /// (x, y) of first spare slot for team 0
    pub spare_start_team0: (f32, f32),
    /// (x, y) of first spare slot for team 1
    pub spare_start_team1: (f32, f32),
    /// (step_x, step_y) between consecutive spare slots
    pub spare_step: (f32, f32),
    /// Number of columns per spare row (pieces wrap after this many)
    pub spare_cols: u32,
    /// Logical canvas width (pixels)
    pub canvas_w: f32,
    /// Logical canvas height (pixels)
    pub canvas_h: f32,
    /// "End Turn" button rectangle
    pub button_end_turn: Rect,
    /// "Undo" button rectangle
    pub button_undo: Rect,
}

impl LayoutConstants {
    /// World-space (x, y) of the top-left corner of a board cell.
    pub fn cell_coords(&self, x: u8, y: u8) -> (f32, f32) {
        let x_pos = x as f32 * CELL_WIDTH + self.shift_x;
        let y_pos = y as f32 * CELL_WIDTH + self.shift_y;
        (x_pos, y_pos)
    }

    /// World-space (x, y) from a Point2.
    pub fn cell_coords_point(&self, point: &Point2) -> (f32, f32) {
        self.cell_coords(point.x, point.y)
    }

    /// Board-cell from world-space position (for mouse hit-testing).
    pub fn coords_to_cell(&self, x_pos: f32, y_pos: f32) -> Point2 {
        let x = ((x_pos - self.shift_x) / CELL_WIDTH) as u8;
        let y = ((y_pos - self.shift_y) / CELL_WIDTH) as u8;
        (x, y).into()
    }

    /// Board cell the mouse is hovering over.
    pub fn cell_hovered(&self, canvas: &Canvas2D) -> Point2 {
        let (mouse_x, mouse_y) = canvas.mouse_position();
        self.coords_to_cell(mouse_x, mouse_y)
    }

    /// Center a sprite inside a board cell.
    pub fn sprite_render_pos(&self, sprite_width: f32, point: &Point2) -> (f32, f32) {
        let (cx, cy) = self.cell_coords(point.x, point.y);
        let shift = (CELL_WIDTH - sprite_width) / 2.0;
        (cx + shift, cy + shift)
    }

    /// Return debug overlay rectangles: `(x, y, w, h, is_blue)`.
    /// `is_blue = true` means the blue color (canvas boundary),
    /// `is_blue = false` means the green color (structural regions).
    pub fn debug_regions(&self) -> Vec<(f32, f32, f32, f32, bool)> {
        let mut regions = Vec::new();
        let bw = CELL_WIDTH * 8.0;
        let bh = CELL_WIDTH * 8.0;

        // ── Canvas outline (blue) ──
        regions.push((0.0, 0.0, self.canvas_w, self.canvas_h, true));

        // ── Board (green) ──
        regions.push((self.shift_x, self.shift_y, bw, bh, false));

        if self.shift_x == 0.0 {
            // ── Portrait ──
            let gap = CELL_WIDTH * PORTRAIT_GAP_FACTOR;
            let board_top = self.shift_y;
            let spare0_bot = ROW_HEIGHT;
            let board_bot = board_top + bh;
            let spare1_top = board_bot + gap;
            let spare1_bot = spare1_top + ROW_HEIGHT;
            let btn_top = spare1_bot + gap;
            let btn_bot = btn_top + PORTRAIT_BTN_HEIGHT;
            let w = self.canvas_w;

            // Spare area team 0 (green) — the top row
            regions.push((0.0, 0.0, w, spare0_bot, false));
            // Gap 0→board (green)
            regions.push((0.0, spare0_bot, w, board_top - spare0_bot, false));
            // Spare area team 1 (green)
            regions.push((0.0, spare1_top, w, spare1_bot - spare1_top, false));
            // Gap board→spare1 (green)
            regions.push((0.0, board_bot, w, spare1_top - board_bot, false));
            // Gap spare1→buttons (green)
            regions.push((0.0, spare1_bot, w, btn_top - spare1_bot, false));
            // Button row (green) — full-width outline even though buttons don't fill it
            regions.push((0.0, btn_top, w, btn_bot - btn_top, false));
            // Gap buttons→text (green)
            regions.push((0.0, btn_bot, w, self.text_y - btn_bot, false));
        } else {
            // ── Landscape ──
            let lw = self.shift_x; // left column width
            let buttons_bot = LANDSCAPE_BTN_TOP + LANDSCAPE_BTN_HEIGHT * 2.0 + LANDSCAPE_BTN_GAP;
            let spare0_top = buttons_bot + LANDSCAPE_SPARE0_GAP;
            let spare0_bot = spare0_top + ROW_HEIGHT * 3.0;
            let spare1_top = spare0_bot + LANDSCAPE_SPARE_GAP;
            let spare1_bot = spare1_top + ROW_HEIGHT * 3.0;

            // Button group (green)
            regions.push((0.0, LANDSCAPE_BTN_TOP, lw, buttons_bot - LANDSCAPE_BTN_TOP, false));
            // Gap buttons→spare0 (green)
            regions.push((0.0, buttons_bot, lw, spare0_top - buttons_bot, false));
            // Spare row 0 (green) — 3 rows
            regions.push((0.0, spare0_top, lw, ROW_HEIGHT * 3.0, false));
            // Gap spare0→spare1 (green)
            regions.push((0.0, spare0_bot, lw, spare1_top - spare0_bot, false));
            // Spare row 1 (green) — 3 rows
            regions.push((0.0, spare1_top, lw, ROW_HEIGHT * 3.0, false));
            // Gap spare1→text (green)
            regions.push((0.0, spare1_bot, lw, self.text_y - spare1_bot, false));
        }

        regions
    }
}

/// Derive full layout from canvas logical size.
/// Pure: same inputs always return the same output.
pub fn compute_layout(canvas_width: f32, canvas_height: f32) -> LayoutConstants {
    if canvas_height > canvas_width {
        compute_portrait_layout(canvas_width, canvas_height)
    } else {
        compute_landscape_layout(canvas_width, canvas_height)
    }
}

// ── Portrait ───────────────────────────────────────────────────────────────

fn compute_portrait_layout(canvas_w: f32, canvas_h: f32) -> LayoutConstants {
    let gap = CELL_WIDTH * PORTRAIT_GAP_FACTOR;

    let board_top = ROW_HEIGHT + gap;
    let board_bot = board_top + CELL_WIDTH * 8.0;
    let spare1_top = board_bot + gap;
    let spare1_bot = spare1_top + ROW_HEIGHT;
    let btn_top = spare1_bot + gap;
    let text_y = btn_top + PORTRAIT_BTN_HEIGHT + gap * 2.0;

    // Two buttons centered side-by-side
    let w = CELL_WIDTH * 8.0; // total board width
    let btn_start = 0.0;

    // Spare step: PORTRAIT_SPARE_COLS pieces across full width
    let spare_step_x = w / PORTRAIT_SPARE_COLS as f32;
    let spare_off = (CELL_WIDTH - PIECE_SCALE) / 2.0;

    LayoutConstants {
        shift_x: 0.0,
        shift_y: board_top,
        text_x: 10.0,
        text_y,
        spare_start_team0: (spare_off, spare_off),
        spare_start_team1: (spare_off, spare1_top + spare_off),
        spare_step: (spare_step_x, 0.0),
        spare_cols: PORTRAIT_SPARE_COLS,
        button_end_turn: Rect::new(btn_start, btn_top, PORTRAIT_BTN0_WIDTH, PORTRAIT_BTN_HEIGHT),
        button_undo: Rect::new(
            btn_start + PORTRAIT_BTN0_WIDTH + PORTRAIT_BTN_GAP,
            btn_top,
            PORTRAIT_BTN1_WIDTH,
            PORTRAIT_BTN_HEIGHT,
        ),
        canvas_w,
        canvas_h,
    }
}

// ── Landscape ──────────────────────────────────────────────────────────────

fn compute_landscape_layout(w: f32, _h: f32) -> LayoutConstants {
    let left_col = w - CELL_WIDTH * 8.0;
    let step_x = left_col / LANDSCAPE_SPARE_COLS as f32;

    // Buttons stacked vertically in the left column
    let btn_w = left_col - LANDSCAPE_BTN_PAD * 2.0;
    let btn_h = LANDSCAPE_BTN_HEIGHT;
    let btn_end_turn = Rect::new(LANDSCAPE_BTN_PAD, LANDSCAPE_BTN_TOP, btn_w, btn_h);
    let undo_top = LANDSCAPE_BTN_TOP + btn_h + LANDSCAPE_BTN_GAP;
    let btn_undo = Rect::new(LANDSCAPE_BTN_PAD, undo_top, btn_w, btn_h);
    let buttons_bot = undo_top + btn_h;

    let spare0_top = buttons_bot + LANDSCAPE_SPARE0_GAP;
    let spare0_bot = spare0_top + ROW_HEIGHT * 3.0;
    let spare1_top = spare0_bot + LANDSCAPE_SPARE_GAP;
    let spare1_bot = spare1_top + ROW_HEIGHT * 3.0;
    let text_y = spare1_bot + LANDSCAPE_TEXT_GAP;
    let spare_off = (CELL_WIDTH - PIECE_SCALE) / 2.0;

    LayoutConstants {
        shift_x: left_col,
        shift_y: 0.0,
        text_x: LANDSCAPE_BTN_PAD,
        text_y,
        spare_start_team0: (spare_off, spare0_top + spare_off),
        spare_start_team1: (spare_off, spare1_top + spare_off),
        spare_step: (step_x, ROW_HEIGHT),
        spare_cols: LANDSCAPE_SPARE_COLS,
        button_end_turn: btn_end_turn,
        button_undo: btn_undo,
        canvas_w: w,
        canvas_h: _h,
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{
        PORTRAIT_CANVAS_W, PORTRAIT_CANVAS_H,
        LANDSCAPE_CANVAS_W, LANDSCAPE_CANVAS_H,
    };

    #[test]
    fn portrait_layout() {
        let l = compute_layout(PORTRAIT_CANVAS_W, PORTRAIT_CANVAS_H);
        assert_eq!(l.shift_x, 0.0, "portrait: shift_x should be 0");
        // spare0 starts offset by half the piece overhang
        let off = (CELL_WIDTH - PIECE_SCALE) / 2.0;
        assert_eq!(l.spare_start_team0, (off, off));
        // spare1 top should be below board + gap
        let board_bot = ROW_HEIGHT + CELL_WIDTH * PORTRAIT_GAP_FACTOR + CELL_WIDTH * 8.0;
        let expected_spare1_top = board_bot + CELL_WIDTH * PORTRAIT_GAP_FACTOR;
        assert!((l.spare_start_team1.1 - (expected_spare1_top + off)).abs() < 0.1);
        // buttons below spare1 (by at least ROW_HEIGHT, minus the piece offset)
        assert!(l.button_end_turn.y > l.spare_start_team1.1 + ROW_HEIGHT * 0.5);
        // text below buttons
        assert!(l.text_y > l.button_end_turn.y + PORTRAIT_BTN_HEIGHT * 0.5);
    }

    #[test]
    fn landscape_layout() {
        let l = compute_layout(LANDSCAPE_CANVAS_W, LANDSCAPE_CANVAS_H);
        assert!(l.shift_x > 0.0, "landscape: shift_x should be > 0");
        // left_col = w - board_width = LANDSCAPE_CANVAS_W - CELL_WIDTH * 8
        let expected_shift_x = LANDSCAPE_CANVAS_W - CELL_WIDTH * 8.0;
        assert!((l.shift_x - expected_shift_x).abs() < 0.1, "shift_x should be {}", expected_shift_x);
        // spare starts below the stacked buttons, offset by half the piece overhang
        let btn_bot = LANDSCAPE_BTN_TOP + LANDSCAPE_BTN_HEIGHT * 2.0 + LANDSCAPE_BTN_GAP;
        let expected_spare0_top = btn_bot + LANDSCAPE_SPARE0_GAP;
        let off = (CELL_WIDTH - PIECE_SCALE) / 2.0;
        assert!((l.spare_start_team0.1 - (expected_spare0_top + off)).abs() < 0.1);
    }

    #[test]
    fn cell_roundtrip() {
        let l = compute_layout(PORTRAIT_CANVAS_W, PORTRAIT_CANVAS_H);
        let (x, y) = l.cell_coords(3, 5);
        let p = l.coords_to_cell(x + 1.0, y + 1.0);
        assert_eq!(p, Point2::new(3, 5));
    }
}
