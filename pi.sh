#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ── Parse flags ───────────────────────────────────────────────────────────────
# --offline  disables all network access inside the container

FLAGS=()
#PASSTHROUGH_ARGS=("/root/.local/bin/pi")
PASSTHROUGH_ARGS=("pi")

for arg in "$@"; do
    case "$arg" in
        --offline) FLAGS+=("--offline") ;;
        --force-rebuild) FLAGS+=("--force-rebuild") ;;
        *)         PASSTHROUGH_ARGS+=("$arg") ;;
    esac
done

DOCKER_ARGS+=(
    # Pi config
    -v "$HOME/.pi/agent/:/root/.pi/agent/"
)

# ── API key ───────────────────────────────────────────────────────────────────
if [[ -n "${ANTHROPIC_API_KEY:-}" ]]; then
   # DOCKER_ARGS+=(-e ANTHROPIC_API_KEY)
    DOCKER_ARGS+=(-e DEEPSEEK_API_KEY)
else
    echo "Warning: ANTHROPIC_API_KEY is not set. Pi may ask you to /login."
    echo "Add 'export ANTHROPIC_API_KEY=sk-ant-...' to your ~/.zshrc to fix this."
fi

echo "${PASSTHROUGH_ARGS[@]}"
./sandbox/run.sh ${FLAGS[@]+"${FLAGS[@]}"} "${DOCKER_ARGS[@]}" -- "${PASSTHROUGH_ARGS[@]}"


