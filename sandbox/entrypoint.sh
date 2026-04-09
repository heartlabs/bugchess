#!/usr/bin/env bash
set -euo pipefail

# Start a virtual display so Playwright can run Chromium in headed mode.
# The display is not visible on the host — it lives entirely inside the container.
# Ignores harmless warnings
Xvfb :99 -screen 0 1280x1024x24 2>/dev/null &
export DISPLAY=:99

# --- Option B: VNC access ---
# Uncomment these lines to expose the virtual display so you can watch browser
# sessions live from your host. Prerequisites:
#   1. Uncomment x11vnc/novnc/websockify in the Dockerfile apt install list.
#   2. Uncomment the -p port mappings in claude.sh.
# Then open http://localhost:6080 in your browser to see the agent's display.
#
# x11vnc -display :99 -forever -shared -nopw -rfbport 5900 &
# websockify 6080 localhost:5900 &

exec "$@"
