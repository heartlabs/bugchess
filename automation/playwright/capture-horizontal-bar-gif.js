#!/usr/bin/env node
/**
 * Capture horizontal-bar merge animation as a GIF using Playwright VIDEO recording.
 *
 * Key insight: page.screenshot() takes ~50-100ms per call, which is far too slow
 * to capture a 400ms merge animation. Playwright's video recorder captures at the
 * browser's actual render rate (~30fps), getting every visual frame.
 *
 * Flow:
 *   1. Start video recording via Playwright context
 *   2. Play through the game interaction (place pieces, trigger merge)
 *   3. Stop recording → get .webm file
 *   4. Use ffmpeg to crop the 3×3 area and export as GIF at 1:1 game speed
 */
const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');
const { chromium } = require('playwright');

const BASE_URL = process.env.BASE_URL || 'http://127.0.0.1:4000/index.htm';
const TIMEOUT_MS = Number(process.env.TIMEOUT_MS || 30000);
const HEADLESS = process.env.HEADLESS !== 'false';

// Board geometry (must match game-render/src/constants.rs)
const CELL = 64 * 1.1875; // CELL_ABSOLUTE_WIDTH
const SHIFT_X = 60;
const SHIFT_Y = 0;

// Crop adjustments (empirically measured for this viewport)
const CROP_ADJUST_X = 54;
const CROP_ADJUST_Y = 10;
const CROP_ADJUST_W = 39;
const CROP_ADJUST_H = 39;

const ROOT = path.join(__dirname, '../..');
const VIDEO_DIR = path.join(ROOT, 'html/gifs/video');
const OUTPUT_GIF = path.join(ROOT, 'html/gifs/horizontal-bar-merge.gif');
const GIF_SIZE = 267;

// Timing plan (all values in ms)
// Setup creates 12 add-unused anims + 2 place anims + turns.
// Team 1's piece at (5,5) swooshes across the board during setup.
// We must wait for ALL of that to finish before marking trim-start.
const SETUP_SETTLE_MS = 4000;
const INITIAL_HOLD_MS = 100;        // briefly show board with single piece at (2,2)
const AFTER_FIRST_PLACE_MS = 200;   // enough to register the click, no need to wait for anim
const AFTER_SECOND_PLACE_MS = 2500; // merge: place(400) + swoosh(400) + spawn + margin
const FINAL_HOLD_MS = 1200;         // hold the finished horizontal bar

function ensureCommand(cmd) {
  const check = spawnSync('bash', ['-lc', `command -v ${cmd}`], { encoding: 'utf8' });
  if (check.status !== 0) throw new Error(`${cmd} not found in PATH`);
}

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function cellCenter(x, y) {
  return {
    x: SHIFT_X + x * CELL + CELL / 2,
    y: SHIFT_Y + y * CELL + CELL / 2,
  };
}

function cropAndConvert(videoPath, cropX, cropY, cropW, cropH, trimStartSeconds, trimDurationSeconds) {
  // Probe video duration
  const probe = spawnSync('ffprobe', [
    '-v', 'error',
    '-show_entries', 'format=duration',
    '-of', 'csv=p=0',
    videoPath,
  ], { encoding: 'utf8' });
  const totalDuration = parseFloat((probe.stdout || '').trim());
  console.log(`Video duration: ${totalDuration.toFixed(2)}s`);

  // Crop the 3×3 area and convert to GIF at native framerate (1:1 with game speed)
  const ff = spawnSync('ffmpeg', [
    '-y', '-v', 'error',
    '-ss', trimStartSeconds.toFixed(3),
    '-t', trimDurationSeconds.toFixed(3),
    '-i', videoPath,
    '-vf', [
      `crop=${cropW}:${cropH}:${cropX}:${cropY}`,
      `scale=${GIF_SIZE}:${GIF_SIZE}:flags=lanczos`,
      'split[s0][s1]',
      '[s0]palettegen=stats_mode=diff[p]',
      '[s1][p]paletteuse=dither=sierra2_4a',
    ].join(','),
    OUTPUT_GIF,
  ], { encoding: 'utf8' });

  if (ff.status !== 0) throw new Error('ffmpeg crop+gif failed: ' + (ff.stderr || ff.stdout));

  // Verify
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

    // Move mouse to safe corner so no hover effects appear
    await page.mouse.move(5, 5);

    // Wait for ALL setup animations to complete:
    // - 12× add-unused anims (~133ms each)
    // - PlacePiece at (2,2) for team 0 (400ms)
    // - NextTurn
    // - PlacePiece at (5,5) for team 1 (400ms) ← this is the "flying piece"
    // - NextTurn back to team 0
    await delay(SETUP_SETTLE_MS);

    // Mark trim start — board is now fully settled, team 0's turn, Place state
    const trimStartSeconds = Number(process.hrtime.bigint() - recordingStartedAt) / 1_000_000_000;
    const trimDurationSeconds =
      (INITIAL_HOLD_MS + AFTER_FIRST_PLACE_MS + AFTER_SECOND_PLACE_MS + FINAL_HOLD_MS) / 1000;

    console.log('Plan:');
    console.log(`  trim_start=${trimStartSeconds.toFixed(3)}s  clip_duration=${trimDurationSeconds.toFixed(3)}s`);
    console.log('  Board: team0 piece at (2,2), team1 piece at (5,5) [outside crop]');
    console.log('  Action 1: hold initial state for ' + INITIAL_HOLD_MS + 'ms');
    console.log('  Action 2: click (1,2) to place, wait ' + AFTER_FIRST_PLACE_MS + 'ms');
    console.log('  Action 3: click (3,2) to place+merge, wait ' + AFTER_SECOND_PLACE_MS + 'ms');
    console.log('  Action 4: hold final merged state for ' + FINAL_HOLD_MS + 'ms');

    // Phase 1: Show initial state — one red piece at center of 3×3 crop
    await delay(INITIAL_HOLD_MS);

    // Phase 2: Place piece at (1,2) — left of the initial piece
    await canvas.click({ position: cellCenter(1, 2) });
    await page.mouse.move(5, 5);
    await delay(AFTER_FIRST_PLACE_MS);

    // Phase 3: Place piece at (3,2) — right of the initial piece → triggers merge
    await canvas.click({ position: cellCenter(3, 2) });
    await page.mouse.move(5, 5);

    // Phase 4: Wait for full merge animation to play out
    await delay(AFTER_SECOND_PLACE_MS);

    // Phase 5: Hold final frame showing the horizontal bar
    await delay(FINAL_HOLD_MS);

    // Compute crop rectangle (same geometry as the cropped screenshot)
    const cropX = Math.round(SHIFT_X + CELL + CROP_ADJUST_X);
    const cropY = Math.round(SHIFT_Y + CELL + CROP_ADJUST_Y);
    const cropW = Math.round(CELL * 3 + CROP_ADJUST_W);
    const cropH = Math.round(CELL * 3 + CROP_ADJUST_H);

    // Close page to finalize video
    const videoPath = await page.video().path();
    await page.close();
    await context.close();

    console.log(`Video saved: ${videoPath}`);
    console.log(`Crop: x=${cropX} y=${cropY} w=${cropW} h=${cropH}`);

    // Convert video → cropped GIF
    cropAndConvert(videoPath, cropX, cropY, cropW, cropH, trimStartSeconds, trimDurationSeconds);
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
