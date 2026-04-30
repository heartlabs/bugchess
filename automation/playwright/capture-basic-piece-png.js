#!/usr/bin/env node
/**
 * Capture a screenshot of the basic piece placed on the board.
 * Uses the same proven patterns as the GIF capture scripts.
 *
 * Output: html/gifs/piece_basic.png (3×3 crop around placed piece)
 */
const fs = require('fs');
const path = require('path');
const { chromium } = require('playwright');

const BASE_URL = process.env.BASE_URL || 'http://127.0.0.1:4000/index.html';
const TIMEOUT_MS = Number(process.env.TIMEOUT_MS || 30000);
const HEADLESS = process.env.HEADLESS === 'true';

const GAME_WIDTH = 1290;
const GAME_HEIGHT = 2520;
const CELL = 161.25;
const SHIFT_X = 0;
const SHIFT_Y = 0.7 * CELL + 0.4 * CELL;

const ROOT = path.join(__dirname, '../..');
const OUTPUT_PNG = path.join(ROOT, 'html/gifs/piece_basic.png');

const SETUP_SETTLE_MS = 4000;
const AFTER_PLACE_MS = 500;
const CROP_BORDER_PX = 2;

function delay(ms) { return new Promise(r => setTimeout(r, ms)); }

async function main() {
    const browser = await chromium.launch({ headless: HEADLESS });
    const context = await browser.newContext({
        viewport: { width: 688, height: 1344 }, deviceScaleFactor: 1,
    });
    const page = await context.newPage();

    try {
        await page.goto(BASE_URL, { waitUntil: 'domcontentloaded', timeout: TIMEOUT_MS });
        await page.addStyleTag({ content: '#game_container { height: 100dvh !important; }' });
        await page.locator('a[onclick="runOffline()"]').click({ timeout: TIMEOUT_MS });
        const canvas = page.locator('#glcanvas');
        await canvas.waitFor({ state: 'visible', timeout: TIMEOUT_MS });

        // Wait for WASM to fully initialize
        await page.waitForFunction(() => {
            const c = document.querySelector('#glcanvas');
            return c && c.width > 300;
        }, { timeout: TIMEOUT_MS });

        const box = await canvas.boundingBox();
        const scale = Math.min(box.width / GAME_WIDTH, box.height / GAME_HEIGHT);
        const leftPad = box.x + (box.width - GAME_WIDTH * scale) / 2;
        const topPad = box.y + (box.height - GAME_HEIGHT * scale) / 2;
        console.log(`Scale: ${scale.toFixed(4)}, pad: (${leftPad.toFixed(1)}, ${topPad.toFixed(1)})`);

        // gameClick helper
        async function gameClick(x, y) {
            await page.mouse.move(x, y);
            await delay(50);
            await page.mouse.down();
            await delay(30);
            await page.mouse.up();
        }

        function cellCenter(x, y) {
            return {
                x: leftPad + (SHIFT_X + x * CELL + CELL / 2) * scale,
                y: topPad + (SHIFT_Y + y * CELL + CELL / 2) * scale,
            };
        }

        await page.mouse.move(5, 5);
        await delay(SETUP_SETTLE_MS);

        // Capture the pre-placed piece at (2,2) — no click needed
        // Compute crop: 3×3 region centered on (2,2) → origin cell (1,1)
        const originX = 1, originY = 1, N = 3;
        const cropLeft = leftPad + (SHIFT_X + originX * CELL) * scale;
        const cropTop = topPad + (SHIFT_Y + originY * CELL) * scale;
        const cropSize = N * CELL * scale;
        const clip = {
            x: Math.round(cropLeft - CROP_BORDER_PX),
            y: Math.round(cropTop - CROP_BORDER_PX),
            width: Math.round(cropSize + 2 * CROP_BORDER_PX),
            height: Math.round(cropSize + 2 * CROP_BORDER_PX),
        };

        console.log(`Clip: x=${clip.x} y=${clip.y} w=${clip.width} h=${clip.height}`);
        await page.screenshot({ path: OUTPUT_PNG, clip });
        console.log(`\nSaved ${OUTPUT_PNG}`);
    } finally {
        await context.close();
        await browser.close();
    }
}

main().catch(e => { console.error(e.message); process.exit(1); });
