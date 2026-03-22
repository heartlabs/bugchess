#!/usr/bin/env node
/**
 * Convert captured PNG frames to animated GIFs
 */

const fs = require('fs');
const path = require('path');
const GIFEncoder = require('gif-encoder');
const PNG = require('pngjs').PNG;

const FRAMES_DIR = '/Users/neidhartorlich/dev/personal/megachess/html/gifs/frames';
const OUTPUT_DIR = '/Users/neidhartorlich/dev/personal/megachess/html/gifs';

async function loadPNG(filePath) {
    return new Promise((resolve, reject) => {
        fs.createReadStream(filePath)
            .pipe(new PNG())
            .on('parsed', function () {
                resolve(this);
            })
            .on('error', reject);
    });
}

async function createGIF(inputFiles, outputName, delay = 100) {
    console.log(`Creating ${outputName}...`);

    try {
        // Load first image to get dimensions
        const firstImage = await loadPNG(inputFiles[0]);
        const width = firstImage.width;
        const height = firstImage.height;

        // Create encoder
        const gif = new GIFEncoder(width, height);
        gif.createReadStream().pipe(fs.createWriteStream(path.join(OUTPUT_DIR, outputName)));

        gif.start();
        gif.setDelay(delay);

        // Add frames, repeating them for smooth animation
        for (const file of inputFiles) {
            const img = await loadPNG(file);

            // Add frame once
            gif.addFrame(img);

            // Add frame again to extend duration (creates pause effect)
            gif.addFrame(img);
            gif.addFrame(img);
        }

        gif.end();

        await new Promise(resolve => setTimeout(resolve, 500));
        const size = fs.statSync(path.join(OUTPUT_DIR, outputName)).size;
        console.log(`✓ ${outputName} (${(size / 1024).toFixed(1)}KB)`);
    } catch (error) {
        console.error(`✗ Error creating ${outputName}:`, error.message);
    }
}

async function main() {
    if (!fs.existsSync(OUTPUT_DIR)) {
        fs.mkdirSync(OUTPUT_DIR, { recursive: true });
    }

    console.log('🎬 Creating animated GIFs from captured frames...\n');

    // Create GIFs from captured frames
    await createGIF([FRAMES_DIR + '/basic_01_placement.png', FRAMES_DIR + '/basic_02_movement.png'], 'piece_basic.gif', 80);
    await createGIF([FRAMES_DIR + '/cross_01_merged.png'], 'piece_cross.gif', 100);
    await createGIF([FRAMES_DIR + '/bar_01_merged.png'], 'piece_bar.gif', 100);
    await createGIF([FRAMES_DIR + '/queen_01_partial.png'], 'piece_queen.gif', 100);

    // Create simple placeholder GIFs for missing pieces
    console.log('\nCreating placeholder GIFs...');

    // Create simple solid color images as placeholders
    const createPlaceholder = async (name) => {
        const png = new PNG({ width: 400, height: 300 });

        // Fill with dark background
        for (let i = 0; i < png.data.length; i += 4) {
            png.data[i] = 26;      // R - dark
            png.data[i + 1] = 26;  // G - dark
            png.data[i + 2] = 26;  // B - dark
            png.data[i + 3] = 255; // A - opaque
        }

        const tempFile = path.join(OUTPUT_DIR, `_temp_${name}.png`);
        await new Promise((resolve, reject) => {
            png.pack().pipe(fs.createWriteStream(tempFile)).on('finish', resolve).on('error', reject);
        });

        await createGIF([tempFile], `piece_${name}.gif`, 100);
        fs.unlinkSync(tempFile);
    };

    await createPlaceholder('sniper');
    await createPlaceholder('castle');

    console.log('\n✅ GIF creation complete!');
    console.log('\nFiles created:');
    fs.readdirSync(OUTPUT_DIR)
        .filter(f => f.endsWith('.gif'))
        .forEach(f => {
            const size = fs.statSync(path.join(OUTPUT_DIR, f)).size;
            console.log(`   ${f} (${(size / 1024).toFixed(1)}KB)`);
        });
}

main().catch(console.error);
