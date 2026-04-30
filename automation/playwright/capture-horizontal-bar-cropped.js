#!/usr/bin/env node
const fs = require('fs');
const { chromium } = require('playwright');
const BASE_URL = process.env.BASE_URL || 'http://127.0.0.1:4000/index.html';
const OUTPUT = process.env.OUTPUT || 'html/gifs/horizontal-bar-after-cropped-3x3.png';
const TIMEOUT_MS = Number(process.env.TIMEOUT_MS || 30000);
const HEADLESS = process.env.HEADLESS !== 'false';

const CELL = 64 * 1.1875 * 2; // CELL_ABSOLUTE_WIDTH = 152
const SHIFT_X = 60 * 2 * 1.5; // PIECE_SCALE = 180
const SHIFT_Y = 0;
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

(async () => {
  const browser = await chromium.launch({ headless: HEADLESS });
  const context = await browser.newContext({
    viewport: { width: 1280, height: 1024 },
    deviceScaleFactor: 1,
  });
  const page = await context.newPage();

  try {
    await page.goto(BASE_URL, { waitUntil: 'domcontentloaded', timeout: TIMEOUT_MS });
    await page.locator('a[onclick="runOffline()"]').click({ timeout: TIMEOUT_MS });

    const canvas = page.locator('#glcanvas');
    await canvas.waitFor({ state: 'visible', timeout: TIMEOUT_MS });
    await page.waitForTimeout(1500);

    await canvas.click({ position: cellCenter(1, 2) });
    await page.waitForTimeout(500);
    await canvas.click({ position: cellCenter(3, 2) });
    await page.waitForTimeout(1300);

    const box = await canvas.boundingBox();
    if (!box) {
      throw new Error('Canvas bounds not available');
    }

    await page.mouse.move(5, 5);
    await page.waitForTimeout(200);

    const clip = {
      x: Math.round(box.x + SHIFT_X + CELL + CROP_ADJUST_X),
      y: Math.round(box.y + SHIFT_Y + CELL + CROP_ADJUST_Y),
      width: Math.round(CELL * 3 + CROP_ADJUST_W),
      height: Math.round(CELL * 3 + CROP_ADJUST_H),
    };

    await page.screenshot({ path: OUTPUT, clip });
    console.log(`Saved ${OUTPUT}`);
    console.log(`Clip: x=${clip.x}, y=${clip.y}, w=${clip.width}, h=${clip.height}`);
  } finally {
    await context.close();
    await browser.close();
  }
})();
