# SESSION_LOG.md

Append-only log for all agent sessions. Each agent's file (`.agents/AGENTS-*.agent.md`) describes when and how to write here.

## 2026-03-31: Claude Opus 4.6 -- Code Review & Bug Fix

- **Bug fixed:** `blast()` and `targeted_shoot()` in `GameController` lacked exhaustion checks. The UI prevented exploits but multiplayer remote commands could bypass it. Added `can_use_special()` guards.
- **Dead code removed:** Unused `exhaustion_clone` variable in `targeted_shoot()`.
- **Tests added:** 2 snapshot tests (0006: HBar merge + blast, 0007: Cross merge + capture) and 2 unit tests verifying exhaustion rejection for blast/shoot.
- **SOUL.md:** Added "Game Mechanics Deep Knowledge" section documenting merge system, combat/range system, exhaustion system, and event architecture.

## 2026-03-31: Claude Opus 4.6 -- Developer Experience Overhaul

- Created `Justfile` with recipes: test, lint, fmt, fmt-check, build, build-wasm, serve, doc
- Added `//!` module-level doc comments to all 5 crate entry points
- Added CI quality gates: `cargo +nightly fmt --check` and `cargo clippy -- -D warnings` to `game-wasm.yml`
- Updated CI actions to current versions, added `Swatinem/rust-cache@v2`
- Fixed clippy warning: added `Default` impl for `UndoManager`
- Deleted dead code: `nakama.rs`, `custom_client.rs`, `output.txt`, nakama docker config
- Fixed window title: "Makrochess" → "Bugchess"
- Refreshed `README.md`: fixed broken GIF paths, added all 6 merge GIFs

## 2026-04-15: Minion -- Matchmaking with matchbox `?next=2`

- **Matchmaking support:** Added `?next=2` query parameter to matchbox URL when room_id is "common" (Find Opponent mode). This enables the matchbox server's rudimentary matching service, creating a new room for every 2 players.
- **Refactored URL building:** Extracted `build_url` helper function with unit tests.
- **Preserved existing behavior:** Create Game mode (random UUID room) and explicit room connections remain unchanged, ensuring reconnection still works.
- **Tests added:** Three unit tests verify URL generation for common room, random room, and rooms with special characters.

## 2026-04-18: Claude Opus 4.6 -- UI Optimization for Player Onboarding

- **UI flow optimized:** Reordered main menu to prioritize "Play with a friend" (formerly Create Game) and demote "Find Opponent" (moved to bottom, darker styling).
- **Combined copy and join:** The separate "Copy Invite Link" and "Join Game" buttons are now a single "Copy Invite & Start Room" button that copies the link and immediately transitions the player into the game board.
- **Updated test automation:** Adjusted Playwright test scripts (`start-game.js`, `webrtc-probe.js`) to reflect new button texts and flow.
- **SOUL.md:** Added note about player onboarding improvements.

## 2026-04-24: Minion — Dual-orientation layout engine (layout.rs + dynamic Canvas2D)

### What changed

Replaced hardcoded global layout constants with a `compute_layout(canvas_w, canvas_h) -> LayoutConstants` pure function (`game-render/src/layout.rs`). Canvas size is chosen at runtime based on actual window orientation. Layout recalculates on orientation flip via polling in the main loop.

### Portrait layout (1080×1800 canvas)

```
 y     ┌──────────────────────────┐
 0     │ Spare row (team 0)       │  ROW_HEIGHT=96, shifted (-20,-20)
       │  20 overlapping pieces   │
 ──────┤                          ├── y≈96
       │ gap = cell × 0.25 = 33.75│
 ──────┤                          ├── y≈130 = board_top
       │                          │
       │  8×8 BOARD              │  cell=135, 8×135=1080 wide
       │  filled to full width   │
       │                          │
 ──────┤                          ├── y≈1210
       │ gap                      │
 ──────┤                          ├── y≈1244
       │ Spare row (team 1)       │  shifted (-20, spare1_top-20)
 ──────┤                          ├── y≈1340
       │ gap                      │
 ──────┤                          ├── y≈1373
       │ [End Turn] [Undo]        │  h=70, buttons start at x=0 (not centered)
 ──────┤                          ├── y≈1511
       │ "Click on..." text      │  text_x=10
       │ "Click target..."       │  2 lines, FONT_SIZE=50
       │                          │
       │ (scroll space)          │  remaining ~289px
 └──────┴──────────────────────────┘ y=1800
```

### Landscape layout (1920×1080 canvas)

```
 x=0                   x=840               x=1920
┌──────────────────────┬──────────────────────────┐
│ [End Turn]           │                          │ y=18, h=84
│ [Undo]               │                          │ y=112, h=84
│                      │                          │
│ Team 0 spare         │      8×8 BOARD           │ spare0: x=0, y=200
│ 3 rows × 7 cols     │      cell=135×135         │ 3×96 = 288px tall
│                      │      shift_x=840         │
│ ──────────────────── │                          │ spare1: x=0, y=520
│ Team 1 spare         │                          │
│ 3 rows × 7 cols     │                          │
│                      │                          │
│ "Click on..."       │                          │ text_x=10, text_y≈878
│ "Click target..."   │                          │
└──────────────────────┴──────────────────────────┘
```

**Left column (840px wide):** buttons stacked vertically (84px each), then 3+3 spare rows with `ROW_HEIGHT=96` (pieces overlap slightly), then text. All spare positions shifted 20px up/left for tighter packing.

### Constants now decoupled

| Constant   | Value | Purpose |
|------------|-------|---------|
| `CELL_WIDTH` | 135.0 | Board cell size (1080÷8) |
| `PIECE_SCALE`| 162.0 | Piece sprite size (independent of layout) |
| `FONT_SIZE` | 50.0  | Text rendering |
| `ROW_HEIGHT`| 96.0  | Spare row spacing (independent of PIECE_SCALE) |

`ROW_HEIGHT` and landscape button height (`LANDSCAPE_BTN_HEIGHT=84`) are standalone constants — changing `PIECE_SCALE` only affects piece visuals, not layout.

### Resize / orientation switching

Main loop polls `screen_width()` vs `screen_height()` each frame. On flip:
1. Creates new `Canvas2D` at the new logical size
2. Calls `state.handle_resize(w, h)` → `compute_layout` → `BoardRender::set_layout` (snaps sprites, preserves animations) → `update_buttons`