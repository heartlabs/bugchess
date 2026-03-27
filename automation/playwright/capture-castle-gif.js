#!/usr/bin/env node
/**
 * Capture castle merge animation as a GIF using Playwright VIDEO recording.
 *
 * Castle pattern (3×3):
 *   Any  Own  Any       (2,2)=Own  ← pre-placed
 *   Own  Free Own       (1,3)=Own  (2,3)=Free  (3,3)=Own
 *   Any  Own  Any       (2,4)=Own
 *
 * Pattern anchored at grid (1,2) so the pre-placed piece at (2,2) fills
 * the top OwnPiece slot. Merged Castle appears at (2,3).
 *
 * Click plan: place (1,3), (3,3), (2,4). Last placement triggers merge.
 * Crop: 3×3 centered on (2,3).
 *
 * IMPORTANT: The game renders to a 900×800 Canvas2D which is scaled/letterboxed
 * to fit the browser viewport (1280×~922). All click positions and crop coordinates
 * must account for this scale + offset transformation.
 */
const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');
const { chromium } = require('playwright');

const BASE_URL = process.env.BASE_URL || 'http://127.0.0.1:4000/index.htm';
const TIMEOUT_MS = Number(process.env.TIMEOUT_MS || 30000);
const HEADLESS = process.env.HEADLESS !== 'false';

// Game internal resolution (Canvas2D in game-main/src/constants.rs)
const GAME_WIDTH = 900;
const GAME_HEIGHT = 800;

// Board geometry (game-render/src/constants.rs)
const CELL = 64 * 1.1875; // CELL_ABSOLUTE_WIDTH = 76
const SHIFT_X = 60;
const SHIFT_Y = 0;

const ROOT = path.join(__dirname, '../..');
const VIDEO_DIR = path.join(ROOT, 'html/gifs/video');
const OUTPUT_GIF = path.join(ROOT, 'html/gifs/castle-merge.gif');
const GIF_SIZE = 267;
const CROP_BORDER_PX = 2; // small aesthetic border around the cells

// Timing plan (all values in ms)
const SETUP_SETTLE_MS = 4000;
const INITIAL_HOLD_MS = 100;
const AFTER_PLACE_MS = 50;         // click fast — placement is the boring part
const AFTER_MERGE_PLACE_MS = 5000; // final placement triggers merge — wait generously
const FINAL_HOLD_MS = 1200;        // hold so viewer can see the finished piece

function ensureCommand(cmd) {
  const check = spawnSync('bash', ['-lc', `command -v ${cmd}`], { encoding: 'utf8' });
  if (check.status !== 0) throw new Error(`${cmd} not found in PATH`);
}

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Convert game-internal cell coordinates to viewport/screen pixel position.
 * Accounts for Canvas2D scaling + letterbox padding.
 */
function makeCellCenter(scale, leftPad, topPad) {
  return function cellCenter(x, y) {
    const gameX = SHIFT_X + x * CELL + CELL / 2;
    const gameY = SHIFT_Y + y * CELL + CELL / 2;
    return {
      x: leftPad + gameX * scale,
      y: topPad + gameY * scale,
    };
  };
}

/**
 * Compute crop rectangle in video space for an NxN cell region.
 * originCellX/Y = top-left cell of the region (in game grid coords).
 */
function computeCrop(N, originCellX, originCellY, scale, leftPad, topPad) {
  const gameLeft = SHIFT_X + originCellX * CELL;
  const gameTop = SHIFT_Y + originCellY * CELL;
  const videoLeft = leftPad + gameLeft * scale;
  const videoTop = topPad + gameTop * scale;
  const videoSize = N * CELL * scale;

  return {
    x: Math.round(videoLeft - CROP_BORDER_PX),
    y: Math.round(videoTop - CROP_BORDER_PX),
    w: Math.round(videoSize + 2 * CROP_BORDER_PX),
    h: Math.round(videoSize + 2 * CROP_BORDER_PX),
  };
}

function cropAndConvert(videoPath, crop, trimStartSeconds, trimDurationSeconds) {
  const probe = spawnSync('ffprobe', [
    '-v', 'error', '-show_entries', 'format=duration', '-of', 'csv=p=0', videoPath,
  ], { encoding: 'utf8' });
  const totalDuration = parseFloat((probe.stdout || '').trim());
  console.log(`Video duration: ${totalDuration.toFixed(2)}s`);

  const ff = spawnSync('ffmpeg', [
    '-y', '-v', 'error',
    '-ss', trimStartSeconds.toFixed(3),
    '-t', trimDurationSeconds.toFixed(3),
    '-i', videoPath,
    '-vf', [
      `crop=${crop.w}:${crop.h}:${crop.x}:${crop.y}`,
      `scale=${GIF_SIZE}:${GIF_SIZE}:flags=lanczos`,
      'split[s0][s1]',
      '[s0]palettegen=stats_mode=diff[p]',
      '[s1][p]paletteuse=dither=sierra2_4a',
    ].join(','),
    OUTPUT_GIF,
  ], { encoding: 'utf8' });

  if (ff.status !== 0) throw new Error('ffmpeg crop+gif failed: ' + (ff.stderr || ff.stdout));

  const info = spawnSync('ffprobe', [
    '-v', 'error', '-count_frames', '-select_streams', 'v:0',
    '-show_entries', 'stream=width,height,nb_read_frames,r_frame_rate',
    '-of', 'default=noprint_wrappers=1',
    OUTPUT_GIF,
  ], { encoding: 'utf8' });

  console.log('GIF info:');
  console.log((info.stdout || '').trim());
}

async function main() {
  ensureCommand('ffmpeg');
  ensureCommand('ffprobe');

  fs.rmSync(VIDEO_DIR, { recursive: true, force: true });
  fs.mkdirSync(VIDEO_DIR, { recursive: true });

  const browser = await chromium.launch({ headless: HEADLESS });
  const context = await browser.newContext({
    viewport: { width: 1280, height: 1024 },
    deviceScaleFactor: 1,
    recordVideo: { dir: VIDEO_DIR, size: { width: 1280, height: 1024 } },
  });
  const page = await context.newPage();
  const recordingStartedAt = process.hrtime.bigint();

  try {
    // Navigate and start offline game
    await page.goto(BASE_URL, { waitUntil: 'domcontentloaded', timeout: TIMEOUT_MS });
    await page.locator('a[onclick="runOffline()"]').click({ timeout: TIMEOUT_MS });

    const canvas = page.locator('#glcanvas');
    await canvas.waitFor({ state: 'visible', timeout: TIMEOUT_MS });

    // Compute Canvas2D scale and padding from actual canvas bounding box
    const box = await canvas.boundingBox();
    const scale = Math.min(box.width / GAME_WIDTH, box.height / GAME_HEIGHT);
    const leftPad = box.x + (box.width - GAME_WIDTH * scale) / 2;
    const topPad = box.y + (box.height - GAME_HEIGHT * scale) / 2;
    console.log(`Canvas: ${box.width}x${box.height} at (${box.x},${box.y})`);
    console.log(`Scale: ${scale.toFixed(6)}, leftPad: ${leftPad.toFixed(1)}, topPad: ${topPad.toFixed(1)}`);

    const cellCenter = makeCellCenter(scale, leftPad, topPad);

    // Move mouse to safe corner
    await page.mouse.move(5, 5);

    // Wait for ALL setup animations
    await delay(SETUP_SETTLE_MS);

    // Mark trim start
    const trimStartSeconds = Number(process.hrtime.bigint() - recordingStartedAt) / 1_000_000_000;
    const trimDurationSeconds =
      (INITIAL_HOLD_MS + 2 * AFTER_PLACE_MS + AFTER_MERGE_PLACE_MS + FINAL_HOLD_MS) / 1000;

    // Log click positions for debugging
    const cc13 = cellCenter(1, 3);
    const cc33 = cellCenter(3, 3);
    const cc24 = cellCenter(2, 4);
    console.log(`\nClick positions (viewport px):`);
    console.log(`  (1,3) → (${cc13.x.toFixed(1)}, ${cc13.y.toFixed(1)})`);
    console.log(`  (3,3) → (${cc33.x.toFixed(1)}, ${cc33.y.toFixed(1)})`);
    console.log(`  (2,4) → (${cc24.x.toFixed(1)}, ${cc24.y.toFixed(1)})`);

    console.log(`\nPlan:`);
    console.log(`  trim_start=${trimStartSeconds.toFixed(3)}s  clip_duration=${trimDurationSeconds.toFixed(3)}s`);
    console.log(`  Action 1: hold initial state for ${INITIAL_HOLD_MS}ms`);
    console.log(`  Action 2: click (1,3) to place left-center, wait ${AFTER_PLACE_MS}ms`);
    console.log(`  Action 3: click (3,3) to place right-center, wait ${AFTER_PLACE_MS}ms`);
    console.log(`  Action 4: click (2,4) to place bottom (triggers merge), wait ${AFTER_MERGE_PLACE_MS}ms`);
    console.log(`  Action 5: hold final merged state for ${FINAL_HOLD_MS}ms`);

    // Phase 1: Show initial state
    await delay(INITIAL_HOLD_MS);

    // Phase 2: Place piece at (1,3)
    await canvas.click({ position: cellCenter(1, 3) });
    await page.mouse.move(5, 5);
    await delay(AFTER_PLACE_MS);

    // Phase 3: Place piece at (3,3)
    await canvas.click({ position: cellCenter(3, 3) });
    await page.mouse.move(5, 5);
    await delay(AFTER_PLACE_MS);

    // Phase 4: Place piece at (2,4) → triggers merge
    await canvas.click({ position: cellCenter(2, 4) });
    await page.mouse.move(5, 5);

    // Phase 5: Wait for merge animation
    await delay(AFTER_MERGE_PLACE_MS);

    // Phase 6: Hold final frame
    await delay(FINAL_HOLD_MS);

    // Compute crop: 3×3 centered on (2,3), origin cell (1,2)
    const crop = computeCrop(3, 1, 2, scale, leftPad, topPad);

    // Close page to finalize video
    const videoPath = await page.video().path();
    await page.close();
    await context.close();

    console.log(`\nVideo saved: ${videoPath}`);
    console.log(`Crop: x=${crop.x} y=${crop.y} w=${crop.w} h=${crop.h}`);

    // Convert video → cropped GIF
    cropAndConvert(videoPath, crop, trimStartSeconds, trimDurationSeconds);
    console.log(`\nSaved ${OUTPUT_GIF}`);
  } catch (err) {
    await page.close().catch(() => {});
    await context.close().catch(() => {});
    throw err;
  } finally {
    await browser.close();
  }
}

main().catch((error) => {
  console.error(`Error: ${error.message}`);
  process.exit(1);
});
