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

## Layout Subsystem (`layout.rs`)

There are two main layout modes based on orientation: portrait (1080×1800) and landscape (1920×1080). The game renders to these logical fixed-size canvases. The Canvas2D struct handles the actual drawing, scaling, and coordinate transformations to the actual screen size. 

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

## Animations (`animation.rs`)

Entry point bottleneck — `start(&self, board_render: &mut BoardRender)` gives access to the layout-owning `BoardRender` (via `board_render.layout`).

### Chain flow
```
Action → BulletAnimation → DieAnimation → RemovePieceAnimation (→ chain continues)
```
Macro-based dispatch. Animations queue in `next_animations` and get pushed to `current_animations` when the previous chain stage finishes.

## CSS / Canvas Sizing (`html/index.htm`)

Only the `#game` container rules affect the canvas:

The canvas element itself is created at the **logical** size (1080×1800 portrait, 1920×1080 landscape) via `Canvas2D::new(w, h)` in `main.rs`. CSS scales it to fit the viewport.

## Validation Checklist

- [ ] `cargo check --all-targets` — 0 errors, 0 warnings
- [ ] `cargo test -p game-render` — all 3 layout tests pass
- [ ] If orientation constants changed: verify both `compute_portrait_layout` and `compute_landscape_layout` produce reasonable values
- [ ] Check main invariant: The board width should always be exactly the smaller screen dimension so the board optimally fills the available space.
- [ ] You can optionally use the playwright screenshot skill to capture the canvas in both orientations and verify visually that layout changes are correct and no elements are misplaced or mis-scaled.
