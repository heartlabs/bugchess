#!/usr/bin/env node
/**
 * Capture real in-game placement/merge screencasts and export piece GIFs.
 * Strategy: deterministic cell clicks + board-only zoom crops around relevant cells.
 */

const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const BASE_URL = process.env.BASE_URL || 'http://127.0.0.1:4000/index.html';
const TIMEOUT_MS = Number(process.env.TIMEOUT_MS || 30000);
const HEADLESS = process.env.HEADLESS !== 'false';

const ROOT_DIR = path.join(__dirname, '../..');
const FRAMES_ROOT = path.join(ROOT_DIR, 'html/gifs/frames');
const OUTPUT_DIR = path.join(ROOT_DIR, 'html/gifs');

const CELL_ABSOLUTE_WIDTH = 64 * 1.1875;
const SHIFT_X = 60;
const SHIFT_Y = 0;
const BOARD_WIDTH = 8;
const BOARD_HEIGHT = 8;

const SCENARIOS = [
  {
    piece: 'bar',
    preTurns: 0,
    placements: [
      { x: 1, y: 2 },
      { x: 3, y: 2 },
    ],
    focus: { minX: 1, maxX: 3, minY: 1, maxY: 3 },
  },
  {
    piece: 'cross',
    preTurns: 0,
    placements: [
      { x: 2, y: 1 },
      { x: 1, y: 2 },
      { x: 3, y: 2 },
      { x: 2, y: 3 },
    ],
    focus: { minX: 1, maxX: 3, minY: 1, maxY: 3 },
  },
  {
    piece: 'basic',
    preTurns: 0,
    placements: [{ x: 3, y: 3 }],
    focus: { minX: 2, maxX: 4, minY: 2, maxY: 4 },
  },
  {
    piece: 'queen',
    preTurns: 0,
    placements: [
      { x: 1, y: 3 },
      { x: 3, y: 3 },
      { x: 0, y: 4 },
      { x: 4, y: 4 },
      { x: 1, y: 5 },
      { x: 3, y: 5 },
      { x: 2, y: 6 },
    ],
    focus: { minX: 0, maxX: 4, minY: 2, maxY: 6 },
  },
  {
    piece: 'sniper',
    preTurns: 0,
    placements: [
      { x: 1, y: 1 },
      { x: 3, y: 1 },
      { x: 1, y: 3 },
      { x: 3, y: 3 },
    ],
    focus: { minX: 1, maxX: 3, minY: 1, maxY: 3 },
  },
  {
    piece: 'castle',
    preTurns: 0,
    placements: [
      { x: 2, y: 1 },
      { x: 1, y: 2 },
      { x: 3, y: 2 },
    ],
    focus: { minX: 1, maxX: 3, minY: 1, maxY: 3 },
  },
];

if (!fs.existsSync(FRAMES_ROOT)) {
  fs.mkdirSync(FRAMES_ROOT, { recursive: true });
}
if (!fs.existsSync(OUTPUT_DIR)) {
  fs.mkdirSync(OUTPUT_DIR, { recursive: true });
}

function ensureCommand(command) {
  const result = spawnSync('bash', ['-lc', `command -v ${command}`], { encoding: 'utf8' });
  if (result.status !== 0) {
    throw new Error(`${command} is required but was not found in PATH`);
  }
}

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function waitForCanvas(page) {
  await page.locator('#glcanvas').waitFor({ state: 'visible', timeout: TIMEOUT_MS });
  await delay(1200);
}

async function startOfflineGame(page) {
  await page.goto(BASE_URL, { waitUntil: 'domcontentloaded', timeout: TIMEOUT_MS });
  const offline = page.getByText('Offline', { exact: true });
  if (await offline.count()) {
    await offline.first().click({ timeout: TIMEOUT_MS });
  } else {
    await page.locator('#select_game').locator('a').first().click({ timeout: TIMEOUT_MS });
  }
  await waitForCanvas(page);
}

function cellCenter(cellX, cellY) {
  return {
    x: SHIFT_X + cellX * CELL_ABSOLUTE_WIDTH + CELL_ABSOLUTE_WIDTH / 2,
    y: SHIFT_Y + cellY * CELL_ABSOLUTE_WIDTH + CELL_ABSOLUTE_WIDTH / 2,
  };
}

async function clickCell(page, cellX, cellY) {
  const { x, y } = cellCenter(cellX, cellY);
  await page.locator('#glcanvas').click({ position: { x, y } });
}

function getBoardOnlyZoomClip(canvasBox, focus) {
  const boardLeft = canvasBox.x + SHIFT_X;
  const boardTop = canvasBox.y + SHIFT_Y;
  const boardWidthPx = CELL_ABSOLUTE_WIDTH * BOARD_WIDTH;
  const boardHeightPx = CELL_ABSOLUTE_WIDTH * BOARD_HEIGHT;

  const padCells = 0.08;
  const minX = Math.max(0, focus.minX - padCells);
  const maxX = Math.min(BOARD_WIDTH, focus.maxX + 1 + padCells);
  const minY = Math.max(0, focus.minY - padCells);
  const maxY = Math.min(BOARD_HEIGHT, focus.maxY + 1 + padCells);

  const x = boardLeft + minX * CELL_ABSOLUTE_WIDTH;
  const y = boardTop + minY * CELL_ABSOLUTE_WIDTH;
  const width = (maxX - minX) * CELL_ABSOLUTE_WIDTH;
  const height = (maxY - minY) * CELL_ABSOLUTE_WIDTH;

  const boardInsetPx = 2;
  const leftGutterSafetyPx = 112;

  const rawX = Math.max(Math.floor(boardLeft + boardInsetPx), Math.floor(x));
  const rawY = Math.max(Math.floor(boardTop + boardInsetPx), Math.floor(y));
  const rawW = Math.min(Math.floor(boardWidthPx - boardInsetPx), Math.floor(width));
  const rawH = Math.min(Math.floor(boardHeightPx - boardInsetPx), Math.floor(height));

  const clippedX = rawX + leftGutterSafetyPx;
  const clippedW = Math.max(120, rawW - leftGutterSafetyPx);

  return {
    x: clippedX,
    y: rawY,
    width: clippedW,
    height: rawH,
  };
}

async function captureBurst(page, clip, frameDir, frameCounter, amount, waitMs) {
  for (let i = 0; i < amount; i += 1) {
    const filename = `${String(frameCounter.value).padStart(4, '0')}.png`;
    await page.screenshot({ path: path.join(frameDir, filename), clip });
    frameCounter.value += 1;
    await delay(waitMs);
  }
}

async function placePiece(page, placement) {
  await clickCell(page, placement.x, placement.y);
  await delay(340);
}

function exportGifFromFrames(frameDir, pieceName) {
  const outputFile = path.join(OUTPUT_DIR, `piece_${pieceName}.gif`);
  const ffmpeg = spawnSync(
    'ffmpeg',
    [
      '-v',
      'error',
      '-y',
      '-framerate',
      '10',
      '-i',
      path.join(frameDir, '%04d.png'),
      '-vf',
      "fps=10,scale=420:420:flags=lanczos,split[s0][s1];[s0]palettegen=stats_mode=single[p];[s1][p]paletteuse=dither=sierra2_4a",
      outputFile,
    ],
    { encoding: 'utf8' }
  );

  if (ffmpeg.status !== 0) {
    throw new Error(`ffmpeg failed for ${pieceName}: ${ffmpeg.stderr || ffmpeg.stdout}`);
  }

  const frameCount = spawnSync(
    'ffprobe',
    [
      '-v',
      'error',
      '-count_frames',
      '-select_streams',
      'v:0',
      '-show_entries',
      'stream=nb_read_frames',
      '-of',
      'csv=p=0',
      outputFile,
    ],
    { encoding: 'utf8' }
  );

  const frames = Number((frameCount.stdout || '').trim());
  if (!Number.isFinite(frames) || frames < 2) {
    throw new Error(`Generated GIF is not animated for ${pieceName}; frame count=${frameCount.stdout}`);
  }

  return { outputFile, frames };
}

async function captureScenario(page, scenario) {
  const { piece, preTurns, placements, focus } = scenario;
  console.log(`\n🎬 Capturing ${piece}...`);
  await startOfflineGame(page);

  if (preTurns > 0) {
    console.log(`Skipping ${preTurns} turn(s) before capture...`);
  }

  const canvasBox = await page.locator('#glcanvas').boundingBox();
  if (!canvasBox) {
    throw new Error('Canvas bounds not available');
  }

  const clip = getBoardOnlyZoomClip(canvasBox, focus);
  const frameDir = path.join(FRAMES_ROOT, piece);
  fs.rmSync(frameDir, { recursive: true, force: true });
  fs.mkdirSync(frameDir, { recursive: true });

  const frameCounter = { value: 0 };

  await captureBurst(page, clip, frameDir, frameCounter, 6, 90);
  for (const placement of placements) {
    await captureBurst(page, clip, frameDir, frameCounter, 4, 70);
    await placePiece(page, placement);
    await captureBurst(page, clip, frameDir, frameCounter, 16, 50);
  }

  await captureBurst(page, clip, frameDir, frameCounter, 16, 70);

  const result = exportGifFromFrames(frameDir, piece);
  console.log(`✓ piece_${piece}.gif (${result.frames} frames)`);
}

async function main() {
  ensureCommand('ffmpeg');
  ensureCommand('ffprobe');

  const browser = await chromium.launch({ headless: HEADLESS });
  const context = await browser.newContext({
    viewport: { width: 1280, height: 1024 },
    deviceScaleFactor: 1,
  });
  const page = await context.newPage();

  try {
    const selectedScenarios = process.env.CAPTURE_ONLY
      ? SCENARIOS.filter((s) => s.piece === process.env.CAPTURE_ONLY)
      : SCENARIOS;

    for (const scenario of selectedScenarios) {
      await captureScenario(page, scenario);
    }

    console.log('\n✅ Piece GIF generation completed.');
  } finally {
    await context.close();
    await browser.close();
  }
}

main().catch((error) => {
  console.error(`❌ ${error.message}`);
  process.exit(1);
});
