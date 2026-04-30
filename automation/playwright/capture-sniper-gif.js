#!/usr/bin/env node
/**
 * Capture sniper merge animation as a GIF using Playwright VIDEO recording.
 *
 * Sniper pattern (3×3 anchored at grid (1,1)):
 *   Own  Any  Own       (1,1)=Own              (3,1)=Own
 *   Any  Own  Any       (2,2)=Own ← pre-placed
 *   Own  Any  Own       (1,3)=Own              (3,3)=Own
 *
 * Click plan: place (1,1), (3,1), (1,3), (3,3). Last triggers merge.
 * Crop: 3×3 centered on (2,2), origin cell (1,1).
 * 4 placements from 5 unused → leaves 1.
 */
const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');
const { chromium } = require('playwright');

// Docker detection – exit with helpful message if inside container
if (fs.existsSync('/.dockerenv')) {
  console.error(`
ERROR: Running inside Docker container. Screen recording will be too slow (≤0.5 fps).

Please run this script locally on your host machine for smooth animations:

  1. Build the game: bash build.sh
  2. Serve HTML locally: python3 -m http.server 4000 --directory html
  3. Run capture script: cd automation/playwright && BASE_URL=http://127.0.0.1:4000/index.html HEADLESS=true node ${__filename}

Exiting.
`);
  process.exit(1);
}

const BASE_URL = process.env.BASE_URL || 'http://127.0.0.1:4000/index.html';
const TIMEOUT_MS = Number(process.env.TIMEOUT_MS || 30000);
const HEADLESS = process.env.HEADLESS === 'true';

const GAME_WIDTH = 1290; // PORTRAIT_CANVAS_W = 430 * 3
const GAME_HEIGHT = 2520; // PORTRAIT_CANVAS_H = 840 * 3
const CELL = 161.25; // CELL_WIDTH = PORTRAIT_CANVAS_W / 8
const SHIFT_X = 0; // portrait: board starts at left edge
const SHIFT_Y = 0.7 * CELL + 0.4 * CELL; // ROW_HEIGHT + gap (portrait board_top)

const ROOT = path.join(__dirname, '../..');
const VIDEO_DIR = path.join(ROOT, 'html/gifs/video');
const OUTPUT_GIF = path.join(ROOT, 'html/gifs/sniper-merge.gif');
const GIF_SIZE = 267;
const CROP_BORDER_PX = 2;

const SETUP_SETTLE_MS = 4000;
const INITIAL_HOLD_MS = 100;
const AFTER_PLACE_MS = 50;         // click fast — placement is the boring part
const AFTER_MERGE_PLACE_MS = 5000; // final placement triggers merge — wait generously
const FINAL_HOLD_MS = 1200;        // hold so viewer can see the finished piece

function ensureCommand(cmd) {
  const check = spawnSync('bash', ['-lc', `command -v ${cmd}`], { encoding: 'utf8' });
  if (check.status !== 0) throw new Error(`${cmd} not found in PATH`);
}
function delay(ms) { return new Promise(r => setTimeout(r, ms)); }

function makeCellCenter(scale, leftPad, topPad) {
  return function (x, y) {
    return {
      x: leftPad + (SHIFT_X + x * CELL + CELL / 2) * scale,
      y: topPad + (SHIFT_Y + y * CELL + CELL / 2) * scale,
    };
  };
}

function computeCrop(N, originCellX, originCellY, scale, leftPad, topPad) {
  const videoLeft = leftPad + (SHIFT_X + originCellX * CELL) * scale;
  const videoTop = topPad + (SHIFT_Y + originCellY * CELL) * scale;
  const videoSize = N * CELL * scale;
  return {
    x: Math.round(videoLeft - CROP_BORDER_PX),
    y: Math.round(videoTop - CROP_BORDER_PX),
    w: Math.round(videoSize + 2 * CROP_BORDER_PX),
    h: Math.round(videoSize + 2 * CROP_BORDER_PX),
  };
}

function cropAndConvert(videoPath, crop, trimStart, trimDuration) {
  const probe = spawnSync('ffprobe', ['-v', 'error', '-show_entries', 'format=duration', '-of', 'csv=p=0', videoPath], { encoding: 'utf8' });
  console.log(`Video duration: ${parseFloat((probe.stdout || '').trim()).toFixed(2)}s`);
  const ff = spawnSync('ffmpeg', [
    '-y', '-v', 'error', '-ss', trimStart.toFixed(3), '-t', trimDuration.toFixed(3), '-i', videoPath,
    '-vf', [
      `crop=${crop.w}:${crop.h}:${crop.x}:${crop.y}`,
      `scale=${GIF_SIZE}:${GIF_SIZE}:flags=lanczos`,
      'fps=30',
      'split[s0][s1]', '[s0]palettegen=stats_mode=diff[p]', '[s1][p]paletteuse=dither=sierra2_4a',
    ].join(','),
    OUTPUT_GIF,
  ], { encoding: 'utf8' });
  if (ff.status !== 0) throw new Error('ffmpeg failed: ' + (ff.stderr || ff.stdout));
  const info = spawnSync('ffprobe', ['-v', 'error', '-count_frames', '-select_streams', 'v:0',
    '-show_entries', 'stream=width,height,nb_read_frames,r_frame_rate', '-of', 'default=noprint_wrappers=1', OUTPUT_GIF], { encoding: 'utf8' });
  console.log('GIF info:\n' + (info.stdout || '').trim());
}

async function main() {
  ensureCommand('ffmpeg'); ensureCommand('ffprobe');
  fs.rmSync(VIDEO_DIR, { recursive: true, force: true });
  fs.mkdirSync(VIDEO_DIR, { recursive: true });

  const browser = await chromium.launch({ headless: HEADLESS });
  const context = await browser.newContext({
    viewport: { width: 688, height: 1344 }, deviceScaleFactor: 1,
    recordVideo: { dir: VIDEO_DIR, size: { width: 688, height: 1344 } },
  });
  const page = await context.newPage();
  const t0 = process.hrtime.bigint();

  try {
    await page.goto(BASE_URL, { waitUntil: 'domcontentloaded', timeout: TIMEOUT_MS });
    // Force integer pixel layout: remove 92dvh so game fills viewport exactly
    await page.addStyleTag({ content: '#game_container { height: 100dvh !important; }' });
    await page.locator('a[onclick="runOffline()"]').click({ timeout: TIMEOUT_MS });
    const canvas = page.locator('#glcanvas');
    await canvas.waitFor({ state: 'visible', timeout: TIMEOUT_MS });

    // Wait for WASM to fully initialize (canvas buffer resizes from default 300×150)
    await page.waitForFunction(() => {
      const c = document.querySelector('#glcanvas');
      return c && c.width > 300;
    }, { timeout: TIMEOUT_MS });

    const box = await canvas.boundingBox();
    const scale = Math.min(box.width / GAME_WIDTH, box.height / GAME_HEIGHT);
    const leftPad = box.x + (box.width - GAME_WIDTH * scale) / 2;
    const topPad = box.y + (box.height - GAME_HEIGHT * scale) / 2;
    console.log(`Scale: ${scale.toFixed(4)}, pad: (${leftPad.toFixed(1)}, ${topPad.toFixed(1)})`);
    const cellCenter = makeCellCenter(scale, leftPad, topPad);

    // Game click helper: move mouse to position, wait one frame for the game
    // engine to register the new position, then click. This avoids a race where
    // macroquad's mouse_position() returns a stale value during click processing.
    async function gameClick(x, y) {
      await page.mouse.move(x, y);
      await delay(50);  // ensure at least one game frame sees the new position
      await page.mouse.down();
      await delay(30);
      await page.mouse.up();
    }

    await page.mouse.move(5, 5);
    await delay(SETUP_SETTLE_MS);

    const trimStart = Number(process.hrtime.bigint() - t0) / 1e9;
    const trimDuration = (INITIAL_HOLD_MS + AFTER_MERGE_PLACE_MS + FINAL_HOLD_MS) / 1000;
    console.log(`trim_start=${trimStart.toFixed(3)}s  duration=${trimDuration.toFixed(3)}s`);

    // Phase 1: Hold initial state
    await delay(INITIAL_HOLD_MS);

    // Phase 2-4: Place (1,1), (3,1), (1,3) — need delay between clicks for game to process
    for (const [x, y] of [[1, 1], [3, 1], [1, 3]]) {
      const c = cellCenter(x, y);
      await gameClick(c.x, c.y);
      await delay(AFTER_PLACE_MS);
    }

    // Phase 5: Place (3,3) — bottom-right → triggers merge
    const cMerge = cellCenter(3, 3);
    await gameClick(cMerge.x, cMerge.y);
    await page.mouse.move(5, 5);
    await delay(AFTER_MERGE_PLACE_MS);

    // Phase 6: Hold final
    await delay(FINAL_HOLD_MS);

    const crop = computeCrop(3, 1, 1, scale, leftPad, topPad);
    const videoPath = await page.video().path();
    await page.close(); await context.close();
    console.log(`Crop: x=${crop.x} y=${crop.y} w=${crop.w} h=${crop.h}`);
    cropAndConvert(videoPath, crop, trimStart, trimDuration);
    console.log(`\nSaved ${OUTPUT_GIF}`);
  } catch (err) {
    await page.close().catch(() => { }); await context.close().catch(() => { });
    throw err;
  } finally { await browser.close(); }
}

main().catch(e => { console.error(e.message); process.exit(1); });
