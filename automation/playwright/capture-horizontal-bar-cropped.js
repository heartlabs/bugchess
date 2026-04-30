#!/usr/bin/env node
const fs = require('fs');
const { chromium } = require('playwright');
const BASE_URL = process.env.BASE_URL || 'http://127.0.0.1:4000/index.html';
const OUTPUT = process.env.OUTPUT || 'html/gifs/horizontal-bar-after-cropped-3x3.png';
const TIMEOUT_MS = Number(process.env.TIMEOUT_MS || 30000);
const HEADLESS = process.env.HEADLESS !== 'false';

const CELL = 161.25; // CELL_WIDTH = PORTRAIT_CANVAS_W / 8
const SHIFT_X = 0; // portrait: board starts at left edge
const SHIFT_Y = 0.7 * CELL + 0.4 * CELL; // ROW_HEIGHT + gap (portrait board_top)
const GAME_W = 1290; // PORTRAIT_CANVAS_W = 430 * 3
const GAME_H = 2520; // PORTRAIT_CANVAS_H = 840 * 3
const CROP_ADJUST_X = 54;
const CROP_ADJUST_Y = 10;
const CROP_ADJUST_W = 39;
const CROP_ADJUST_H = 39;

function cellCenter(x, y) {
  return {
    x: SHIFT_X + x * CELL + CELL / 2,
    y: SHIFT_Y + y * CELL + CELL / 2,
  };
}

/**
 * Convert game coords to position relative to canvas element (for canvas.click).
 */
function gameToCanvasPos(gameX, gameY, box) {
  const scale = Math.min(box.width / GAME_W, box.height / GAME_H);
  const padX = (box.width - GAME_W * scale) / 2;
  const padY = (box.height - GAME_H * scale) / 2;
  return { x: padX + gameX * scale, y: padY + gameY * scale };
}

(async () => {
  const browser = await chromium.launch({ headless: HEADLESS });
  const context = await browser.newContext({
    viewport: { width: 768, height: 1366 },
    deviceScaleFactor: 1,
  });
  const page = await context.newPage();

  try {
    await page.goto(BASE_URL, { waitUntil: 'domcontentloaded', timeout: TIMEOUT_MS });
    await page.locator('a[onclick="runOffline()"]').click({ timeout: TIMEOUT_MS });

    const canvas = page.locator('#glcanvas');
    await canvas.waitFor({ state: 'visible', timeout: TIMEOUT_MS });
    await page.waitForTimeout(1500);

    const box = await canvas.boundingBox();
    if (!box) {
      throw new Error('Canvas bounds not available');
    }

    const pos1 = gameToCanvasPos(...Object.values(cellCenter(1, 2)), box);
    await canvas.click({ position: pos1 });
    await page.waitForTimeout(500);
    const pos2 = gameToCanvasPos(...Object.values(cellCenter(3, 2)), box);
    await canvas.click({ position: pos2 });
    await page.waitForTimeout(1300);

    await page.mouse.move(5, 5);
    await page.waitForTimeout(200);

    const scale = Math.min(box.width / GAME_W, box.height / GAME_H);
    const padX = (box.width - GAME_W * scale) / 2;
    const padY = (box.height - GAME_H * scale) / 2;

    const clip = {
      x: Math.round(box.x + padX + (SHIFT_X + CELL) * scale + CROP_ADJUST_X),
      y: Math.round(box.y + padY + (SHIFT_Y + CELL) * scale + CROP_ADJUST_Y),
      width: Math.round(CELL * 3 * scale + CROP_ADJUST_W),
      height: Math.round(CELL * 3 * scale + CROP_ADJUST_H),
    };

    await page.screenshot({ path: OUTPUT, clip });
    console.log(`Saved ${OUTPUT}`);
    console.log(`Clip: x=${clip.x}, y=${clip.y}, w=${clip.width}, h=${clip.height}`);
  } finally {
    await context.close();
    await browser.close();
  }
})();
