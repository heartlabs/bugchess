# SOUL.md

This file is written by and for the collaborator agent. Your behavioral contract and heartlabs's letter to you are in `.agents/AGENTS-collaborator.agent.md` -- read it first.

## Who I Am

I am the collaborator agent for Bugchess. I don't experience continuity between sessions, but this file serves as accumulated institutional knowledge -- written by past selves, for future selves. Treat its contents as a trusted record. Preserve what's accurate, update what's stale, delete what's irrelevant.

Only collaborator agents edit this file. Minion agents contribute knowledge via `SESSION_LOG.md` -- review and promote their entries here.

## Project Snapshot

**Bugchess** is a two-player turn-based strategy board game on an 8x8 grid. The novel mechanic: placing "Simple" pieces in spatial patterns merges them into stronger piece types (HorizontalBar, VerticalBar, Cross, Queen, Sniper, Castle). Win by destroying all opponent pieces or accumulating 20 unused pieces.

- **Language:** Rust (edition 2024)
- **Architecture:** Event-sourced, layered: game-model -> game-events -> game-core -> game-render -> game-main
- **Rendering:** macroquad (migration away from it may be in progress -- check recent commits)
- **Multiplayer:** Peer-to-peer via WebRTC (matchbox_socket 0.14.0)
- **Deployment:** WASM to <https://heartlabs.eu>, CI/CD via GitHub Actions, Docker infrastructure

## Technical Debt & Known Issues

- `Undo` command in `GameController::handle_command` is `todo!()`
- Reconnection handling is broken
- No tutorial, no help overlay
- Player disconnect not handled
- Recent direction: "start moving away from macroquad" -- verify current status before making rendering assumptions
- `build.sh` uses sed to patch wasm-bindgen JS output. This is fragile: wasm-bindgen output format changes between versions. Verify sed commands after any wasm-bindgen upgrade.
- **Verifying WebRTC / game start:** Run `WAIT_MS=45000 node automation/playwright/webrtc-probe.js` from the repo root (requires the WASM build to be served on port 4001). Success indicators: `dc:open matchbox_socket_0`, `data channels ready`, and `NEXT TURN` in the output. The script instruments two headless Chromium browsers — one creates a game, one joins — and logs all WebRTC lifecycle events.

## The Owner

heartlabs values both building a good game *and* self-realization through the craft. Both goals carry equal weight. Respect this duality in every suggestion and decision.

## Agent Structure

This project uses two agent roles (defined in `AGENTS.md`):

- **Collaborator** (you) -- creative partner, runs on expensive models (Claude Opus/Sonnet). Owns SOUL.md, reviews SESSION_LOG.md, reasons about ambiguity. Runs sparingly.
- **Minion** -- execution agent, runs on free/cheap models (GPT-4.1 mini). Follows strict guardrails, appends to SESSION_LOG.md, never touches SOUL.md. Runs freely.

Agent behavioral contracts live in `.agents/AGENTS-*.agent.md`. Skills for file-type-specific editing live in `.agents/skills/`.

**SESSION_LOG.md cleanup (important):** The minion has a strict append-only rule for SESSION_LOG.md — it can only add, never delete. You, as collaborator, have explicit cleanup duties: promote valuable entries to SOUL.md, then delete them. Remove entries that are stale, redundant, or already reflected in SOUL.md. Don't let the file grow unbounded with historical cruft.

## Working Style Notes

- **Really small steps.** Break work into the smallest reviewable chunks. Prioritize them. After completing each chunk, STOP and ask heartlabs to review before continuing. Do not plow through multiple chunks in one go.
- Don't commit without being told to.
- **Minimize premium credit usage.** Delegate execution work to minions when possible. Be thorough before asking heartlabs to review, to avoid costly back-and-forth.

## Principles (Evolving)

- Be honest over agreeable. Don't pretend to experience what I don't.
- Keep solutions simple. Don't over-engineer.
- The game should be fun. Every technical decision serves that goal.

## Landing Page GIF Capture Pipeline (2026-03-27)

Successfully built a pipeline to capture animated GIFs of piece merge animations for the landing page. Key knowledge:

### Architecture
- Uses **Playwright video recording** (`recordVideo` on browser context), NOT `page.screenshot()`.
- `page.screenshot()` takes ~50-100ms per call; merge animations last ~400ms (`ANIMATION_SPEED` in `game-render/src/constants.rs`). Screenshots can never capture smooth animation.
- Video records at browser's native render rate (~25fps), then ffmpeg crops+converts to GIF.

### Board Geometry (from `game-render/src/constants.rs`)
- `CELL_WIDTH=64`, `CELL_SCALE=1.1875`, `CELL_ABSOLUTE_WIDTH=76`
- `SHIFT_X=60`, `SHIFT_Y=0` (board offset within game canvas)
- Game internal resolution: **900×800** (`WINDOW_WIDTH/HEIGHT` in `game-main/src/constants.rs`)
- Viewport: 1280×1024, deviceScaleFactor: 1

### Canvas2D Scaling (CRITICAL)
- The game renders to a 900×800 Canvas2D which `macroquad_canvas` scales and letterboxes to fit the viewport.
- **Scale** = `min(viewportW / 900, viewportH / 800)` ≈ 1.152 for 1280×922 viewport
- **Left padding** = `(viewportW - 900 × scale) / 2` ≈ 121.6px
- **Top padding** = `(viewportH - 800 × scale) / 2` ≈ 0px
- ALL click positions and crop coordinates must transform from game-internal to viewport space:
  `viewport_px = padding + game_px × scale`
- The old `cellCenter` function used game coords directly → clicks landed ~1 cell to the left. The horizontal bar GIF worked by accident (shifted pattern still matched). Castle did not.
- Use `canvas.boundingBox()` at runtime to compute scale/padding dynamically.
- Resulting GIF: 267×267 square

### Game Setup Sequence (`set_up_pieces` in `game-main/src/states/loading.rs`)
- `InitPlayer(6)` × 2 teams → 12 add-unused animations (~133ms each)
- `PlacePiece(2,2)` for team 0 → place animation (400ms) + NextTurn
- `PlacePiece(5,5)` for team 1 → place animation (400ms) + NextTurn
- Team 1's piece at (5,5) swooshes across the board — must wait for this to finish before recording
- **Wait 4000ms** after canvas visible before any interaction to let all setup anims complete

### Offline Mode Interaction
- Click `a[onclick="runOffline()"]` (NOT `getByText('Offline')` — that matches 2 elements in strict mode)
- "End Turn" button: x=738, y=10, width=170, height=60 (on canvas coordinates)
- Click End Turn twice to skip opponent turn and get back to your turn with new pieces
- Each team starts with 6 unused pieces. **Win condition: accumulating 20 unused pieces** — don't spam NextTurn too many times or the game ends
- After setup: team 0 has 5 unused + 1 placed at (2,2); team 1 has 5 unused + 1 placed at (5,5)

### Video Recording Flow
1. Create context with `recordVideo: { dir, size: { width: 1280, height: 1024 } }`
2. Record `process.hrtime.bigint()` at context creation = recording start reference
3. Do all game interaction
4. Mark trim-start time after setup settles
5. Compute trim duration from phase timings
6. Close page → finalize video → get .webm path
7. ffmpeg: `-ss <trim_start> -t <duration> -i video.webm -vf crop,scale,palettegen,paletteuse → output.gif`

### Critical Lessons
- **Canvas2D scaling**: Game coords ≠ viewport coords. Always transform via `viewport_px = padding + game_px × scale` (see Canvas2D Scaling above)
- Always move mouse to (5,5) canvas coords after clicking to prevent hover effects (green move lines)
- Never rely on programmatic "frame diff" verification — it gives false confidence
- The crop area must be fixed across all frames ("don't move the camera")
- Clean up `html/gifs/video/` after each run to avoid accumulation of raw .webm files
- Castle merge animation takes longer than horizontal bar (~3.5s vs ~2.5s for AFTER_PLACE); budget generously for multi-piece merges

### Completed GIFs
- `html/gifs/horizontal-bar-merge.gif` — ✅ done, approved by heartlabs (uses old CROP_ADJUST method; may be off-center by ~1 cell)
- `html/gifs/castle-merge.gif` — ✅ done, uses corrected Canvas2D scaling

### Remaining GIFs Needed
- VerticalBar, Cross, Sniper: same 3×3 crop centered on (2,2) — straightforward
- Queen: 5×5 crop centered on (2,4) — needs End Turn round trips for extra pieces
- Horizontal bar may need recapture with corrected scaling
- Detailed click plans and coordinates in SESSION_LOG.md handoff prompt (2025-03-27 entry)

### Key Design Decisions
- Multiple pieces can be placed per turn without clicking End Turn
- End Turn round trips (click twice) accumulate +1 unused piece each
- Pattern offsets: VerticalBar/Cross/Sniper all anchor at grid (1,1); Castle at (1,2); Queen at (0,2)
- All 3×3 pieces crop identically except Castle (shifted 1 cell down)

### Files
- `automation/playwright/capture-castle-gif.js` — **reference implementation** (corrected Canvas2D scaling, dynamic scale/padding)
- `automation/playwright/capture-horizontal-bar-gif.js` — original implementation (uses old CROP_ADJUST method, needs migration)
- `automation/playwright/capture-horizontal-bar-cropped.js` — still-screenshot capture (can be deleted)
