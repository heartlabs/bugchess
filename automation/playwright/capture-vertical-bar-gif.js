#!/usr/bin/env node
/**
 * Capture vertical-bar merge animation as a GIF using Playwright VIDEO recording.
 *
 * VerticalBar pattern (3×3 anchored at grid (1,1)):
 *   Free Own  Free      (2,1)=Own
 *   Free Own  Free      (2,2)=Own ← pre-placed
 *   Free Own  Free      (2,3)=Own
 *
 * Click plan: place (2,1), then (2,3). Last triggers merge.
 * Crop: 3×3 centered on (2,2), origin cell (1,1).
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
  3. Run capture script: cd automation/playwright && BASE_URL=http://127.0.0.1:4000/index.htm HEADLESS=true node ${__filename}

Exiting.
`);
  process.exit(1);
}

const BASE_URL = process.env.BASE_URL || 'http://127.0.0.1:4000/index.htm';
const TIMEOUT_MS = Number(process.env.TIMEOUT_MS || 30000);
const HEADLESS = process.env.HEADLESS !== 'false';

const GAME_WIDTH = 1800;
const GAME_HEIGHT = 1600;
const CELL = 64 * 1.1875 * 2; // CELL_ABSOLUTE_WIDTH = 152
const SHIFT_X = 60 * 2 * 1.5; // PIECE_SCALE = 180
const SHIFT_Y = 0;

const ROOT = path.join(__dirname, '../..');
const VIDEO_DIR = path.join(ROOT, 'html/gifs/video');
const OUTPUT_GIF = path.join(ROOT, 'html/gifs/vertical-bar-merge.gif');
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
  return function(x, y) {
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
  const probe = spawnSync('ffprobe', ['-v','error','-show_entries','format=duration','-of','csv=p=0',videoPath], {encoding:'utf8'});
  console.log(`Video duration: ${parseFloat((probe.stdout||'').trim()).toFixed(2)}s`);
  const ff = spawnSync('ffmpeg', [
    '-y','-v','error','-ss',trimStart.toFixed(3),'-t',trimDuration.toFixed(3),'-i',videoPath,
    '-vf', [
      `crop=${crop.w}:${crop.h}:${crop.x}:${crop.y}`,
      `scale=${GIF_SIZE}:${GIF_SIZE}:flags=lanczos`,
      'fps=30',
      'split[s0][s1]','[s0]palettegen=stats_mode=diff[p]','[s1][p]paletteuse=dither=sierra2_4a',
    ].join(','),
    OUTPUT_GIF,
  ], {encoding:'utf8'});
  if (ff.status !== 0) throw new Error('ffmpeg failed: ' + (ff.stderr || ff.stdout));
  const info = spawnSync('ffprobe', ['-v','error','-count_frames','-select_streams','v:0',
    '-show_entries','stream=width,height,nb_read_frames,r_frame_rate','-of','default=noprint_wrappers=1',OUTPUT_GIF], {encoding:'utf8'});
  console.log('GIF info:\n' + (info.stdout||'').trim());
}

async function main() {
  ensureCommand('ffmpeg'); ensureCommand('ffprobe');
  fs.rmSync(VIDEO_DIR, { recursive: true, force: true });
  fs.mkdirSync(VIDEO_DIR, { recursive: true });

  const browser = await chromium.launch({ headless: HEADLESS });
  const context = await browser.newContext({
    viewport: { width: 1280, height: 1024 }, deviceScaleFactor: 1,
    recordVideo: { dir: VIDEO_DIR, size: { width: 1280, height: 1024 } },
  });
  const page = await context.newPage();
  const t0 = process.hrtime.bigint();

  try {
    await page.goto(BASE_URL, { waitUntil: 'domcontentloaded', timeout: TIMEOUT_MS });
    await page.locator('a[onclick="runOffline()"]').click({ timeout: TIMEOUT_MS });
    const canvas = page.locator('#glcanvas');
    await canvas.waitFor({ state: 'visible', timeout: TIMEOUT_MS });

    const box = await canvas.boundingBox();
    const scale = Math.min(box.width / GAME_WIDTH, box.height / GAME_HEIGHT);
    const leftPad = box.x + (box.width - GAME_WIDTH * scale) / 2;
    const topPad = box.y + (box.height - GAME_HEIGHT * scale) / 2;
    console.log(`Scale: ${scale.toFixed(4)}, pad: (${leftPad.toFixed(1)}, ${topPad.toFixed(1)})`);
    const cellCenter = makeCellCenter(scale, leftPad, topPad);

    await page.mouse.move(5, 5);
    await delay(SETUP_SETTLE_MS);

    const trimStart = Number(process.hrtime.bigint() - t0) / 1e9;
    const trimDuration = (INITIAL_HOLD_MS + AFTER_MERGE_PLACE_MS + FINAL_HOLD_MS) / 1000;
    console.log(`trim_start=${trimStart.toFixed(3)}s  duration=${trimDuration.toFixed(3)}s`);

    // Phase 1: Hold initial state
    await delay(INITIAL_HOLD_MS);

    // Phase 2: Place (2,1) — top — fast click via mouse API
    const c21 = cellCenter(2, 1);
    await page.mouse.click(c21.x, c21.y);

    // Phase 3: Place (2,3) — bottom → triggers merge
    const c23 = cellCenter(2, 3);
    await page.mouse.click(c23.x, c23.y);
    await page.mouse.move(5, 5);
    await delay(AFTER_MERGE_PLACE_MS);

    // Phase 4: Hold final
    await delay(FINAL_HOLD_MS);

    const crop = computeCrop(3, 1, 1, scale, leftPad, topPad);
    const videoPath = await page.video().path();
    await page.close(); await context.close();
    console.log(`Crop: x=${crop.x} y=${crop.y} w=${crop.w} h=${crop.h}`);
    cropAndConvert(videoPath, crop, trimStart, trimDuration);
    console.log(`\nSaved ${OUTPUT_GIF}`);
  } catch (err) {
    await page.close().catch(()=>{}); await context.close().catch(()=>{});
    throw err;
  } finally { await browser.close(); }
}

main().catch(e => { console.error(e.message); process.exit(1); });
