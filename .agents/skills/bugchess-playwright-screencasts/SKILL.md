```skill
---
name: bugchess-playwright-screencasts
description: Use this skill to capture animated GIFs or video clips of in-game animations (merges, attacks, movement) using Playwright video recording and ffmpeg. Covers coordinate mapping, timing, cropping, and conversion.
user-invocable: true
disable-model-invocation: false
---

# SKILL: Bugchess Playwright Screencast Capture

## Purpose
Capture animated GIFs (or video clips) of in-game events by recording Playwright browser sessions and post-processing with ffmpeg. This is the proven approach — do NOT use `page.screenshot()` for animations (too slow at ~50-100ms/call).

## Prerequisites
- Depends on **bugchess-playwright-screenshots** skill for build, serve, and Playwright setup.
- Requires `ffmpeg` and `ffprobe` in PATH.
- Existing capture scripts live in `automation/playwright/capture-*-gif.js`. Study them as templates.

## The Critical Coordinate Mapping

**This is the #1 gotcha.** The game renders to an internal Canvas2D whose resolution differs from the browser viewport. You MUST map coordinates.

### How It Works
1. The game has an internal resolution defined in `game-main/src/constants.rs` (`WINDOW_WIDTH`, `WINDOW_HEIGHT`).
2. `macroquad_canvas` scales and letterboxes this to fit the actual browser viewport.
3. Board cell positions are defined in `game-render/src/constants.rs` (`CELL_WIDTH`, `CELL_SCALE`, `SHIFT_X`, `SHIFT_Y`).

### How to Map (do this at runtime, never hardcode)
```javascript
const box = await canvas.boundingBox();
const GAME_W = /* read from game-main/src/constants.rs WINDOW_WIDTH */;
const GAME_H = /* read from game-main/src/constants.rs WINDOW_HEIGHT */;
const scale = Math.min(box.width / GAME_W, box.height / GAME_H);
const leftPad = box.x + (box.width - GAME_W * scale) / 2;
const topPad = box.y + (box.height - GAME_H * scale) / 2;

// Game-internal coords → viewport click position:
function gameToViewport(gameX, gameY) {
  return { x: leftPad + gameX * scale, y: topPad + gameY * scale };
}

// Cell center in viewport coords:
const CELL = /* CELL_WIDTH * CELL_SCALE from constants.rs */;
const SHIFT_X = /* from constants.rs */;
const SHIFT_Y = /* from constants.rs */;
function cellCenter(cellX, cellY) {
  const gx = SHIFT_X + cellX * CELL + CELL / 2;
  const gy = SHIFT_Y + cellY * CELL + CELL / 2;
  return gameToViewport(gx, gy);
}
```

**If you skip this mapping, clicks will land ~1 cell off and crops will be wrong.**

## Recording Flow

1. **Create browser context with video recording:**
   ```javascript
   const context = await browser.newContext({
     viewport: { width: 1280, height: 1024 },
     deviceScaleFactor: 1,
     recordVideo: { dir: VIDEO_DIR, size: { width: 1280, height: 1024 } },
   });
   ```

2. **Record a time reference** at context creation: `const t0 = process.hrtime.bigint();`

3. **Navigate and enter game** (use bugchess-playwright-screenshots skill for setup).

4. **Wait for setup animations** — the game runs setup automatically (placing initial pieces, swoosh animations). Wait generously (currently ~4000ms) before interacting. This value may change if game setup changes.

5. **Mark trim-start** just before the interesting action begins:
   ```javascript
   const trimStart = Number(process.hrtime.bigint() - t0) / 1e9;
   ```

6. **Perform game interactions** (clicks, delays). Key rules:
   - **Move mouse to (5,5) after every click** to prevent hover effects (green lines).
   - **Click fast for "boring" setup** (~50ms between non-final clicks).
   - **Wait generously after the final click** that triggers the animation you want to capture (~5000ms for 3×3 merges, more for larger patterns).
   - **Hold the final state** for ≥1 second so the viewer can register what happened.

7. **Close page** to finalize video, then **crop + convert with ffmpeg:**
   ```javascript
   // Crop coordinates also use the scale/padding mapping
   const crop = computeCrop(N, originCellX, originCellY, scale, leftPad, topPad);
   // ffmpeg pipeline: crop → scale → palette → GIF
   ```

## Timing Guidelines

- `ANIMATION_SPEED` is defined in `game-render/src/constants.rs` — check its current value.
- Merge animations involve: place animation + swoosh to center + spawn of new piece. Budget at least `3 × ANIMATION_SPEED` plus margin.
- More pieces merging = longer animation. A 5-piece merge (e.g., Cross) needs more time than a 3-piece merge (e.g., Bar).
- When in doubt, record longer and trim. A too-short clip that cuts before the result is visible is useless.

## Crop Computation

```javascript
function computeCrop(N, originCellX, originCellY, scale, leftPad, topPad) {
  const BORDER = 2; // small aesthetic border
  const videoLeft = leftPad + (SHIFT_X + originCellX * CELL) * scale;
  const videoTop = topPad + (SHIFT_Y + originCellY * CELL) * scale;
  const videoSize = N * CELL * scale;
  return {
    x: Math.round(videoLeft - BORDER),
    y: Math.round(videoTop - BORDER),
    w: Math.round(videoSize + 2 * BORDER),
    h: Math.round(videoSize + 2 * BORDER),
  };
}
```

## ffmpeg Conversion

```bash
ffmpeg -y -v error \
  -ss <trim_start> -t <duration> \
  -i video.webm \
  -vf "crop=W:H:X:Y,scale=SIZE:SIZE:flags=lanczos,split[s0][s1];[s0]palettegen=stats_mode=diff[p];[s1][p]paletteuse=dither=sierra2_4a" \
  output.gif
```

## Pitfalls & Lessons Learned

1. **Never use `page.screenshot()` for animations.** It's 50-100ms per call; animations are ~400ms. Use video recording.
2. **Never trust programmatic frame-diff verification.** Always visually inspect the GIF.
3. **Canvas2D scaling is the #1 source of bugs.** If clicks seem to land in the wrong cell, you probably forgot the coordinate mapping.
4. **The game's setup sequence places pieces with animations.** One piece may swoosh across the entire board. Wait for it to finish before starting your recording window.
5. **Hover effects**: The game draws green lines when the mouse hovers over a cell. Always move the mouse to a safe corner (5,5) after every click.
6. **End Turn accumulation**: If you need more unused pieces than the game provides at start, click the End Turn button (in game coordinates) twice per round trip. But beware: the game ends at 20 unused pieces.
7. **Offline mode selector**: Use `a[onclick="runOffline()"]` — `getByText('Offline')` matches 2 elements.

## Validation
- [ ] GIF file exists and has >10 frames (use `ffprobe -count_frames`).
- [ ] GIF dimensions match expected crop size.
- [ ] **Visually inspect the GIF** — this is the only reliable validation.
- [ ] Clean up `html/gifs/video/` after each run.

## Reference Implementation
Study `automation/playwright/capture-castle-gif.js` as the canonical template. It demonstrates all patterns: coordinate mapping, timing, crop computation, and ffmpeg conversion.
```
