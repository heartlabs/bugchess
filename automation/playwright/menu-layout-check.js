#!/usr/bin/env node
const { chromium } = require('playwright');

const BASE_URL = process.env.BASE_URL || 'http://127.0.0.1:4000/index.htm';
const TIMEOUT_MS = Number(process.env.TIMEOUT_MS || 30000);
const HEADLESS = process.env.HEADLESS !== 'false';

function assertWithMessage(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function overlaps(a, b) {
  return !(
    a.x + a.width <= b.x ||
    b.x + b.width <= a.x ||
    a.y + a.height <= b.y ||
    b.y + b.height <= a.y
  );
}

async function main() {
  const browser = await chromium.launch({ headless: HEADLESS });
  const page = await browser.newPage({ viewport: { width: 1400, height: 900 } });

  try {
    console.log(`Opening page: ${BASE_URL}`);
    await page.goto(BASE_URL, { waitUntil: 'domcontentloaded', timeout: TIMEOUT_MS });

    const game = page.locator('#game');
    const menuItems = page.locator('#select_game .gameMenu a');

    await game.waitFor({ state: 'visible', timeout: TIMEOUT_MS });
    await menuItems.first().waitFor({ state: 'visible', timeout: TIMEOUT_MS });

    const gameBox = await game.boundingBox();
    const itemCount = await menuItems.count();
    assertWithMessage(!!gameBox, 'Could not read #game bounding box');
    assertWithMessage(itemCount >= 3, `Expected at least 3 menu items, got ${itemCount}`);

    const boxes = [];
    for (let i = 0; i < itemCount; i += 1) {
      const item = menuItems.nth(i);
      const box = await item.boundingBox();
      const text = (await item.innerText()).trim();
      assertWithMessage(!!box, `Menu item ${i} (${text}) has no bounding box`);

      assertWithMessage(box.x >= gameBox.x, `Menu item '${text}' is left of game canvas`);
      assertWithMessage(
        box.x + box.width <= gameBox.x + gameBox.width,
        `Menu item '${text}' is right of game canvas`
      );
      assertWithMessage(box.y >= gameBox.y, `Menu item '${text}' is above game canvas`);
      assertWithMessage(
        box.y + box.height <= gameBox.y + gameBox.height,
        `Menu item '${text}' is below game canvas`
      );

      assertWithMessage(box.height >= 44, `Menu item '${text}' height too small (${box.height.toFixed(1)}px)`);
      assertWithMessage(box.width >= 160, `Menu item '${text}' width too small (${box.width.toFixed(1)}px)`);

      boxes.push({ ...box, text });
    }

    for (let i = 0; i < boxes.length; i += 1) {
      for (let j = i + 1; j < boxes.length; j += 1) {
        assertWithMessage(!overlaps(boxes[i], boxes[j]), `Menu items overlap: '${boxes[i].text}' and '${boxes[j].text}'`);
      }
    }

    await page.screenshot({ path: 'automation/playwright/menu-layout-check.png', fullPage: true });
    console.log('Layout check passed: all menu items are inside the game canvas, readable, and non-overlapping.');
  } finally {
    await browser.close();
  }
}

main().catch((error) => {
  console.error(`Layout check failed: ${error.message}`);
  process.exitCode = 1;
});
