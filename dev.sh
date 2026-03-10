#!/usr/bin/env bash
set -euo pipefail

# 1. Build
bash build.sh

# 2. Serve the html/ directory with basic-http-server
PORT=4000
basic-http-server html/ --addr 0.0.0.0:$PORT &
SERVER_PID=$!

echo "Serving at http://localhost:$PORT"

# 3. Open index.htm in the default browser
sleep 1
open "http://localhost:$PORT/index.htm"

# Wait for the server and clean up on exit
trap "kill $SERVER_PID 2>/dev/null" EXIT
wait $SERVER_PID
