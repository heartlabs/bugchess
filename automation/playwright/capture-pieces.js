#!/usr/bin/env node
/**
 * Piece Capture Script for Bugchess
 * 
 * This script helps capture gameplay footage for each piece type.
 * It starts the game in offline mode and pauses at key moments.
 * 
 * Usage:
 *   BASE_URL=http://localhost:4000/index.htm node capture-pieces.js
 * 
 * Then use your preferred screen recording tool (QuickTime, OBS, etc.) 
 * to record the gameplay and convert to GIF with ffmpeg:
 *   ffmpeg -i recording.mov -vf "fps=10,scale=320:-1" piece_cross.gif
 */

const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

const BASE_URL = process.env.BASE_URL || 'http://127.0.0.1:4000/index.htm';
const TIMEOUT_MS = 30000;
const OUTPUT_DIR = 'piece-captures';

// Create output directory for screenshots
if (!fs.existsSync(OUTPUT_DIR)) {
    fs.mkdirSync(OUTPUT_DIR, { recursive: true });
}

async function waitForCanvas(page, label) {
    const canvas = page.locator('#glcanvas');
    await canvas.waitFor({ state: 'visible', timeout: TIMEOUT_MS });
    console.log(`${label}: canvas is visible`);
}

async function saveScreenshot(page, filename, description) {
    await page.screenshot({
        path: path.join(OUTPUT_DIR, filename),
        fullPage: false
    });
    console.log(`✓ Captured: ${description} -> ${filename}`);
}

async function main() {
    const browser = await chromium.launch({ headless: false });
    const context = await browser.newContext({ viewport: { width: 1280, height: 1024 } });
    const page = await context.newPage();

    try {
        console.log('Starting Bugchess piece capture script...');
        console.log(`Opening: ${BASE_URL}`);

        await page.goto(BASE_URL, { waitUntil: 'domcontentloaded', timeout: TIMEOUT_MS });

        // Click Offline to start a game
        console.log('\n📍 Starting offline game...');
        await page.getByText('Offline', { exact: true }).click({ timeout: TIMEOUT_MS });

        await waitForCanvas(page, 'game');

        // Wait for initial board to render
        await page.waitForTimeout(2000);
        await saveScreenshot(page, '00_initial_board.png', 'Initial board state');

        console.log('\n🎮 Game is ready for manual recording.');
        console.log('Use QuickTime (macOS), OBS, or ScreenFlow to record gameplay:');
        console.log('  1. Record piece placement and merging mechanics');
        console.log('  2. Show each piece type being created (Cross, Bar, Queen, Sniper, Castle)');
        console.log('  3. Demonstrate special abilities (click piece twice)');
        console.log('\n📹 When ready, record with: ffmpeg -f gdigrab -i desktop -c:v libx264 -pix_fmt yuv420p output.mp4');
        console.log('🎬 Convert to GIF: ffmpeg -i recording.mov -vf "fps=10,scale=400:-1" output.gif');
        console.log('\n⏸️  Browser window will stay open. Close it when done recording.');

        // Keep browser open for manual interaction
        await new Promise(() => { }); // Never resolves - wait indefinitely

    } catch (error) {
        console.error(`Error: ${error.message}`);
        process.exitCode = 1;
    } finally {
        await context.close();
        await browser.close();
    }
}

main().catch(console.error);
