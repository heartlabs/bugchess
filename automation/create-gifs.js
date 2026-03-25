#!/usr/bin/env node
/**
 * GIF generation entrypoint.
 * Delegates to create-gifs.sh in the same folder.
 */

const { spawnSync } = require('child_process');
const path = require('path');

const scriptPath = path.join(__dirname, 'create-gifs.sh');
const result = spawnSync('bash', [scriptPath], { stdio: 'inherit' });

if (result.error) {
    console.error(`❌ Failed to run ${scriptPath}: ${result.error.message}`);
    process.exit(1);
}

process.exit(result.status ?? 1);
