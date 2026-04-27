---
name: game-canvas-presentation
description: Use this skill when adjusting any visual aspect of the Bugchess canvas — layout, board/piece rendering, animations, sprite scaling, UI buttons, canvas sizing, or orientation handling. Covers everything inside the canvas and the CSS rules that position it.
user-invocable: true
disable-model-invocation: false
---

# SKILL: Bugchess Canvas Presentation

## Purpose
Guide safe edits to the in-canvas rendering pipeline — layout, board rendering, piece sprites, animations, UI buttons, and canvas sizing in CSS. Maintains the architecture of computed layout (no mutable globals), orientation switching, and decoupled constants.

## Architecture Overview

```
main loop (main.rs)
  │  screen_width/height → orientation detection
  │  Canvas2D::new(logical_w, logical_h)
  │
  ├─ state.handle_resize(w, h)
  │     ├─ compute_layout(w, h) → LayoutConstants   (layout.rs)
  │     ├─ BoardRender::set_layout(&new_layout)      (rendering.rs)
  │     └─ CustomRenderContext::update_buttons(&new_layout)  (rendering.rs)
  │
  └─ state.draw() / state.update()
        ├─ BoardRender renders board + placed pieces
        ├─ CustomRenderContext renders buttons + text
        └─ AnimationExpert drives sprite animations via macros
```

All render and layout related code is in the `game-render` crate. The `game-main` crate owns the game loop and canvas creation, but delegates all layout and rendering logic to `game-render`. The HTML/CSS only handles canvas sizing and centering.

## Canvas Dimensions (`game-render/src/constants.rs`)

The logical canvas dimensions are defined as constants at the top of `constants.rs`. From these, `CELL_WIDTH` is derived:

```rust
pub const CELL_WIDTH: f32 = {
    // Smaller dimension / 8 ensures the 8×8 board fills the smaller dimension
    let min_dim = min(PORTRAIT_CANVAS_W, PORTRAIT_CANVAS_H);
    min_dim / 8.0
};
```

The board width is always exactly the smaller canvas dimension (`8 × CELL_WIDTH`). All layout constants are expressed as fractions of `CELL_WIDTH` (e.g., `0.5 * CELL_WIDTH`) — rounded to one decimal place.

All layout constants scale automatically since they're derived from `CELL_WIDTH`.

**Critical invariant**: `CELL_WIDTH` must remain a global `pub const`. Do NOT make it orientation-dependent or a runtime value — it is referenced by `cell_coords()`, `sprite_render_pos()`, `LayoutConstants` methods, sprite animations (`animation.rs`), rendering code (`rendering.rs`), and sprite positioning (`sprite.rs`).

### Landscape canvas height

The landscape canvas height **must** be at least `CELL_WIDTH * 8.0` to fit the full board vertically. The current value satisfies this:

```rust
pub const LANDSCAPE_CANVAS_H: f32 = 430.0 * 3.;  // = 1290.0 ≥ CELL_WIDTH * 8
```

If `CELL_WIDTH` changes (e.g. via portrait dimension changes), verify `LANDSCAPE_CANVAS_H` is still large enough. The board starts at `y=0` in landscape with `shift_y = 0.0`.

## Layout Subsystem (`layout.rs`)

There are two main layout modes based on orientation: portrait (`PORTRAIT_CANVAS_W × PORTRAIT_CANVAS_H`) and landscape (`LANDSCAPE_CANVAS_W × LANDSCAPE_CANVAS_H`). The game renders to these logical fixed-size canvases. The `Canvas2D` struct handles the actual drawing, scaling, and coordinate transformations to the actual screen size.

Each layout mode has a different arrangement of the board, spare pieces, buttons, and text. The `LayoutConstants` struct stores all computed positions and dimensions for these elements, which are calculated in `compute_portrait_layout` and `compute_landscape_layout`.

The main layout calculations are based on the `CELL_WIDTH`. This ensures that if for example the `PIECE_SCALE` changes, the layout will remain unchanged which ensures that you can freely adjust piece visuals without worrying about layout breakage.

The board width should always be exactly the smaller screen dimension so the board optimally fills the available space, and all other elements should be positioned based on that.

### Layout constants reference

Each named constant controls a specific visual relationship. Located in `layout.rs` unless noted.

| Element | Controlling constant | Notes |
|---------|---------------------|-------|
| Cell size | `CELL_WIDTH` (`constants.rs`) | = canvas smaller dimension / 8 |
| Piece visual size | `PIECE_SCALE` (`constants.rs`) | Decoupled from layout — safe to change freely |
| Spare row height | `ROW_HEIGHT` (`constants.rs`) | Standalone — does not depend on PIECE_SCALE |
| Font size | `FONT_SIZE` (`constants.rs`) | Standalone |
| Portrait gap between regions | `PORTRAIT_GAP_FACTOR` | Fraction of CELL_WIDTH |
| Portrait button height | `PORTRAIT_BTN_HEIGHT` | |
| Portrait button widths | `PORTRAIT_BTN0_WIDTH`, `PORTRAIT_BTN1_WIDTH` | |
| Portrait button-to-button gap | `PORTRAIT_BTN_GAP` | |
| Portrait spare columns count | `PORTRAIT_SPARE_COLS` | |
| Landscape first-button top | `LANDSCAPE_BTN_TOP` | |
| Landscape button height | `LANDSCAPE_BTN_HEIGHT` | |
| Landscape button padding | `LANDSCAPE_BTN_PAD` | Horizontal from left edge |
| Landscape button-to-button gap | `LANDSCAPE_BTN_GAP` | |
| Landscape buttons-to-spare gap | `LANDSCAPE_SPARE0_GAP` | |
| Landscape spare groups gap | `LANDSCAPE_SPARE_GAP` | |
| Landscape spare-to-text gap | `LANDSCAPE_TEXT_GAP` | |
| Landscape spare columns count | `LANDSCAPE_SPARE_COLS` | |
| Spare start offset | `SPARE_SHIFT` | Applied as (-shift, -shift) in both orientations |
| Animation timing | `ANIMATION_SPEED` etc. (`constants.rs`) | |

### Portrait layout
```
 x=0                                           x=canvas_w
┌─────────────────────────────────────────────────────┐
│ Spare row team 0                                   │ shifted toward top-left
├─────────────────────────────────────────────────────┤
│                                                     │
│                                                     │
│      8×8 BOARD                                      │ board width = min dimension
│                                                     │
│                                                     │
├─────────────────────────────────────────────────────┤
│ Spare row team 1                                   │ shifted toward top-left
├─────────────────────────────────────────────────────┤
│   [End Turn]                    [Undo]             │ side by side
├─────────────────────────────────────────────────────┤
│ "Click on..." / "Click target..."                   │ description text
│                                                     │ free space below
└─────────────────────────────────────────────────────┘
```

### Landscape layout
```
 x=0              x=left_col              x=canvas_w
┌────────────────┬─────────────────────────────┐
│ [End Turn]     │                             │
│ [Undo]         │    8×8 BOARD                │ board width = min dimension
│                │                             │
│ Team 0 spare   │                             │
│ Team 1 spare   │                             │
│                │                             │
│ "Click on..."  │                             │
└────────────────┴─────────────────────────────┘
```

## Board & Piece Rendering (`rendering.rs`)

### BoardRender (sole layout owner)
- Created with `BoardRender::new(&LayoutConstants, ...)`
- `set_layout(&LayoutConstants)` — called on resize, snaps placed sprites, rebuilds unused sprites, snapshots special sprites, preserves active animation timelines
- `get_layout() -> &LayoutConstants` — cross-crate accessor used by CoreGameState
- Renders the 8×8 board grid, placed pieces, unused pieces, and animation sprites
- If any other struct needs layout info, it must be passed an immutable reference to the `LayoutConstants` from `BoardRender::get_layout()`

### CustomRenderContext (no layout storage)
- `update_buttons(&LayoutConstants)` — rebuilds button rects on resize
- `draw_text(&str, x, y, font_size, color)` — renders description text
- Renders buttons and text in the left column (landscape) or below board (portrait)

### Key rendering helpers on LayoutConstants
- `cell_coords(x, y)` — world-space top-left of a board cell
- `cell_coords_point(point)` — same from a Point2
- `coords_to_cell(x, y)` — hit-test mouse to board cell
- `cell_hovered(canvas)` — board cell under mouse
- `sprite_render_pos(sprite_width, point)` — center a sprite in a board cell

## Sprite System (`sprite.rs`)

- `SpriteRender::new_at_point(x, y, w, h, color, &LayoutConstants)` — positions a sprite on the board
- `SpriteRender::for_piece(piece, &LayoutConstants)` — creates a piece sprite from game state
- `SpriteRender::scale(&self, sprite_width)` — scales sprite, uses `CELL_WIDTH` internally, does **not** need LayoutConstants
- `SpriteRender::move_towards(&self, target_pos, dt, &LayoutConstants)` — smooth movement
- `scale_animation_point(anim_point, &layout)` — maps animation frames to board positions
- Local const `SPRITE_WIDTH = CELL_WIDTH * 0.75` for internal sprite sizing
- `render_pos()` field replaced by `layout.sprite_render_pos(layout, point)`

## Patterns Infographic (`game-main/src/states/core_game_state.rs`)

The patterns overlay is toggled by the **Patterns** button (or `P` key). It renders mini-grid cards showing each pattern a set of own pieces can form, and the resulting piece type. The function `draw_patterns()` handles all rendering.

### Pattern order

Pattern cards are displayed in the order returned by `Pattern::all_patterns()` in `game-model/src/pattern.rs`. Current display order:

1. **Queen** (5×5 plus-shape grid)
2. **HorizontalBar** (3×3 row of three own pieces)
3. **VerticalBar** (3×3 column of three own pieces)
4. **Cross** (3×3 plus-shape with center-free)
5. **Sniper** (3×3 X-shape)
6. **Castle** (3×3 diamond-shape)

### Color coding

| Component | Render style |
|-----------|-------------|
| `OwnPiece` | Filled green (`rgb(60, 200, 60)`)
| `Free` | Filled white
| `Any` | Outlined gray (`rgb(160, 160, 160)`)

Every cell also has a thin semi-transparent dark border (`rgb(60, 60, 60, 160)`) drawn on top, ensuring the grid is always visible regardless of fill color.

### Result piece sprite rotation

Result piece sprites use `SpriteRender::piece_sprite_rect()` to select the atlas rect. The `HorizontalBar` result sprite is rotated by `1.57` rad (90°) to match how the game displays it on the board (see `SpriteRender::for_piece` in `sprite.rs`). `VerticalBar` uses the same atlas rect but is not rotated.

### Layout constants reference

Located in `game-render/src/constants.rs`:

| Constant | Description |
|----------|-------------|
| `PATTERN_CELL_SIZE` | Size of one cell in the mini-grid (`0.28 × CELL_WIDTH`)
| `PATTERN_ELEMENT_GAP` | Gap between grid and result sprite (`0.04 × CELL_WIDTH`)
| `PATTERN_ROW_GAP` | Vertical gap between pattern card rows (`0.60 × CELL_WIDTH`)
| `PATTERN_COL_GAP` | Horizontal gap between pattern card columns (`1.30 × CELL_WIDTH`)
| `PATTERN_PIECE_SIZE` | Size of the result piece sprite (`0.55 × CELL_WIDTH`)

The legend row is drawn above the cards using a scaled-down cell (`PATTERN_CELL_SIZE × 0.6`) and label font (`FONT_SIZE × 0.5`).

## Animations (`animation.rs`)

Entry point bottleneck — `start(&self, board_render: &mut BoardRender)` gives access to the layout-owning `BoardRender` (via `board_render.layout`).

### Chain flow
```
Action → BulletAnimation → DieAnimation → RemovePieceAnimation (→ chain continues)
```
Macro-based dispatch. Animations queue in `next_animations` and get pushed to `current_animations` when the previous chain stage finishes.

## CSS / Canvas Sizing (`html/index.htm`)

### Required viewport meta tag

The `<head>` **must** include this or `100vw` will use a virtual 980px viewport on mobile, making everything ~44% scale in portrait:

```html
<meta name="viewport" content="width=device-width, initial-scale=1.0">
```

### CSS aspect-ratio approach

The `#game` container uses `aspect-ratio` to control on-screen sizing. No fixed height — height is derived from `100vw × aspect-ratio`, so the canvas is always the correct shape and the documentation below remains scrollable.

**The CSS `aspect-ratio` does NOT need to match the logical canvas aspect ratio.** `Canvas2D::draw()` uses `get_size_and_padding()` from the macroquad-canvas library to letterbox the logical canvas inside whatever screen area it receives. Black bars are added as needed. This means:

- Portrait CSS `aspect-ratio` should match the portrait logical canvas ratio (430/840) for minimal letterboxing in portrait
- Landscape CSS `aspect-ratio` (`932/387`) is wider than the landscape logical canvas ratio (`2796/1290 = 2.167`) — `Canvas2D` letterboxes, adding black bars on the sides
- You can change landscape canvas dimensions without touching CSS, as long as the board fits vertically

**However**, the portrait CSS `aspect-ratio` should still match `PORTRAIT_CANVAS_W / PORTRAIT_CANVAS_H` to avoid excessive letterboxing in the primary orientation.

The `#glcanvas` fills `#game` at 100% width/height. The `resize()` callback in `gl.js` sets the WebGL canvas resolution to `canvas.clientWidth × dpr`, so `screen_width()`/`screen_height()` return CSS pixel dimensions. `Canvas2D::draw()` then provides its own letterboxing via `get_size_and_padding()`.

## Validation Checklist

- [ ] `cargo check --all-targets` — 0 errors, 0 warnings
- [ ] `cargo test -p game-render` — all 3 layout tests pass
- [ ] If orientation constants changed: verify both `compute_portrait_layout` and `compute_landscape_layout` produce reasonable values
- [ ] **If canvas constants changed**: verify `LANDSCAPE_CANVAS_H ≥ CELL_WIDTH * 8.0` so the board fits vertically in landscape
- [ ] **If portrait canvas constants changed**: the portrait CSS `aspect-ratio` should be updated to match for minimal letterboxing
- [ ] **If landscape canvas constants changed**: no CSS change needed — `Canvas2D` handles letterboxing, but verify the board fits vertically
- [ ] Check main invariant: The board width should always be exactly the smaller screen dimension so the board optimally fills the available space.
- [ ] You can optionally use the playwright screenshot skill to capture the canvas in both orientations and verify visually that layout changes are correct and no elements are misplaced or mis-scaled.
