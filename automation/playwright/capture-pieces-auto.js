#!/usr/bin/env node
/**
 * Automated Piece Capture Script for Bugchess
 * 
 * This script automatically plays the game and captures frames
 * for each piece type to create animated GIFs.
 */

const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

const BASE_URL = process.env.BASE_URL || 'http://127.0.0.1:4000/index.htm';
const TIMEOUT_MS = 30000;
const OUTPUT_DIR = path.join(__dirname, '../../html/gifs/frames');

// Create output directory for frames
if (!fs.existsSync(OUTPUT_DIR)) {
    fs.mkdirSync(OUTPUT_DIR, { recursive: true });
}

async function waitForCanvas(page, label) {
    const canvas = page.locator('#glcanvas');
    await canvas.waitFor({ state: 'visible', timeout: TIMEOUT_MS });
    console.log(`${label}: canvas is visible`);
}

async function captureFrame(page, filename, description) {
    const framePath = path.join(OUTPUT_DIR, filename);
    await page.screenshot({ path: framePath, fullPage: false });
    console.log(`✓ ${description}`);
    return framePath;
}

async function delay(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

async function main() {
    const browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({
        viewport: { width: 1280, height: 1024 }
    });
    const page = await context.newPage();

    try {
        console.log('🎮 Starting automated piece capture...\n');

        await page.goto(BASE_URL, { waitUntil: 'domcontentloaded', timeout: TIMEOUT_MS });

        // Start offline game
        console.log('📍 Starting offline game...');
        await page.locator('#select_game').locator('a').first().click({ timeout: TIMEOUT_MS });

        await waitForCanvas(page, 'game');
        await delay(2000);

        console.log('\n🐛 Capturing Basic Piece...');
        // Capture basic piece placement and movement
        const canvas = page.locator('#glcanvas');
        const box = await canvas.boundingBox();

        // Click to place a basic piece (clicking on an empty square)
        // Board appears to be roughly in the canvas center
        const boardCenterX = box.x + box.width / 2;
        const boardCenterY = box.y + box.height / 2;

        // Try to place a piece on the board
        await page.click(`canvas`);
        await delay(100);

        // Wait a bit and capture
        await delay(800);
        await captureFrame(page, 'basic_01_placement.png', 'Basic piece after placement');

        await delay(300);

        // Try moving the piece (click and move)
        await page.click('canvas', { x: boardCenterX - 100, y: boardCenterY });
        await delay(300);
        await page.click('canvas', { x: boardCenterX - 50, y: boardCenterY });
        await delay(600);
        await captureFrame(page, 'basic_02_movement.png', 'Basic piece after movement');

        // Continue playing to create formations...
        console.log('\n✚ Setting up Cross pattern (5 pieces in + shape)...');

        // Place pieces to form a cross
        const placements = [
            { x: boardCenterX - 50, y: boardCenterY - 100, label: 'top' },
            { x: boardCenterX - 50, y: boardCenterY - 50, label: 'center-top' },
            { x: boardCenterX - 50, y: boardCenterY, label: 'center' },
            { x: boardCenterX - 100, y: boardCenterY, label: 'left' },
            { x: boardCenterX, y: boardCenterY, label: 'right' },
        ];

        for (let i = 0; i < placements.length; i++) {
            const p = placements[i];
            await page.click('canvas', { x: p.x, y: p.y });
            await delay(400);
            if (i === placements.length - 1) {
                // Last placement should trigger merge
                await delay(800);
                await captureFrame(page, 'cross_01_merged.png', 'Cross piece formed from merge');
            }
        }

        console.log('\n— Setting up Bar pattern (3 pieces in a row)...');

        // Place pieces to form a horizontal bar
        const barPlacements = [
            { x: boardCenterX + 100, y: boardCenterY - 150, label: 'left' },
            { x: boardCenterX + 150, y: boardCenterY - 150, label: 'center' },
            { x: boardCenterX + 200, y: boardCenterY - 150, label: 'right' },
        ];

        for (let i = 0; i < barPlacements.length; i++) {
            const p = barPlacements[i];
            await page.click('canvas', { x: p.x, y: p.y });
            await delay(400);
            if (i === barPlacements.length - 1) {
                await delay(800);
                await captureFrame(page, 'bar_01_merged.png', 'Bar piece formed from merge');
            }
        }

        console.log('\n🔷 Setting up Queen pattern (8 pieces in diamond)...');

        // Place pieces to form a diamond/queen
        const queenPlacements = [
            { x: boardCenterX - 200, y: boardCenterY + 50 },
            { x: boardCenterX - 150, y: boardCenterY + 50 },
            { x: boardCenterX - 150, y: boardCenterY + 100 },
            { x: boardCenterX - 150, y: boardCenterY },
            { x: boardCenterX - 100, y: boardCenterY },
            { x: boardCenterX - 100, y: boardCenterY + 50 },
            { x: boardCenterX - 200, y: boardCenterY },
            { x: boardCenterX - 200, y: boardCenterY + 100 },
        ];

        for (let i = 0; i < Math.min(queenPlacements.length, 6); i++) {
            const p = queenPlacements[i];
            await page.click('canvas', { x: p.x, y: p.y });
            await delay(400);
        }

        await delay(1000);
        await captureFrame(page, 'queen_01_partial.png', 'Queen pattern (partial setup for demonstration)');

        console.log('\n✓ Frame capture complete!');
        console.log(`📁 Frames saved to: ${OUTPUT_DIR}`);
        console.log('\n📹 Next step: Convert frames to GIFs using ffmpeg');
        console.log('   Example: ffmpeg -framerate 10 -i basic_%02d.png -c:v libx264 -pix_fmt yuv420p basic.gif');

    } catch (error) {
        console.error(`❌ Error: ${error.message}`);
        process.exitCode = 1;
    } finally {
        await context.close();
        await browser.close();
    }
}

main().catch(console.error);
