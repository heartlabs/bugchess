#!/usr/bin/env bash
set -euo pipefail

# 1. Build
# Fast dev-wasm profile by default (defined in Cargo.toml).
# Override with: BUILD_PROFILE=release ./dev.sh
BUILD_PROFILE=dev-wasm bash build.sh

# 2. Serve the html/ directory with basic-http-server
PORT=4000
basic-http-server html/ --addr 0.0.0.0:$PORT &
SERVER_PID=$!

echo "Serving at http://localhost:$PORT"

# 3. Open index.html in the default browser
sleep 1
open "http://localhost:$PORT/index.html"

# Wait for the server and clean up on exit
trap "kill $SERVER_PID 2>/dev/null" EXIT
wait $SERVER_PID
