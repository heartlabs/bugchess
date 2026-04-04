#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ── Resolve wasm-bindgen version from Cargo.lock ──────────────────────────────
# The sandbox image bakes in the exact wasm-bindgen-cli version from Cargo.lock.
# The image is tagged with that version so a new image is built automatically
# when the version changes. Old images accumulate; prune with: docker image prune
WASM_BINDGEN_VERSION=$(grep -A1 'name = "wasm-bindgen"' "$SCRIPT_DIR/Cargo.lock" \
    | grep version | head -1 | cut -d'"' -f2)

IMAGE_TAG="bugchess-sandbox:wbg-${WASM_BINDGEN_VERSION}"

# ── Build image if it doesn't exist ───────────────────────────────────────────
if ! docker image inspect "$IMAGE_TAG" > /dev/null 2>&1; then
    echo "Building sandbox image $IMAGE_TAG (first run or wasm-bindgen version changed)..."
    docker build \
        --build-arg WASM_BINDGEN_VERSION="$WASM_BINDGEN_VERSION" \
        -t "$IMAGE_TAG" \
        "$SCRIPT_DIR/sandbox"
fi

# ── Parse flags ───────────────────────────────────────────────────────────────
# --offline  disables all network access inside the container
OFFLINE=0
PASSTHROUGH_ARGS=()
for arg in "$@"; do
    case "$arg" in
        --offline) OFFLINE=1 ;;
        *)         PASSTHROUGH_ARGS+=("$arg") ;;
    esac
done

# ── Assemble docker run arguments ─────────────────────────────────────────────
DOCKER_ARGS=(--rm -it)

if [[ "$OFFLINE" -eq 1 ]]; then
    DOCKER_ARGS+=(--network none)
fi

DOCKER_ARGS+=(
    # Repo: mounted so edits made inside the container are visible on the host
    -v "$SCRIPT_DIR:/workspace"
    # Claude config + credentials: shared with the host claude installation.
    # .claude.json lives in $HOME (not inside .claude/), so both must be mounted.
    -v "$HOME/.claude.json:/root/.claude.json"
    -v "$HOME/.claude:/root/.claude"
    # Cargo caches: named volumes so downloaded crates survive container teardown
    -v "bugchess-cargo-registry:/root/.cargo/registry"
    -v "bugchess-cargo-git:/root/.cargo/git"
    # Compiled artifacts: separate named volume to avoid host/container platform
    # mismatch. The host target/ dir is unused when running inside the container.
    -v "bugchess-target:/cargo-target"
    -e CARGO_TARGET_DIR=/cargo-target
)

# ── API key ───────────────────────────────────────────────────────────────────
# Claude Code stores its session in the macOS Keychain, which is inaccessible
# from inside the container. Pass ANTHROPIC_API_KEY as an env var instead.
# Add it to your shell rc (e.g. ~/.zshrc): export ANTHROPIC_API_KEY=sk-ant-...
if [[ -n "${ANTHROPIC_API_KEY:-}" ]]; then
    DOCKER_ARGS+=(-e ANTHROPIC_API_KEY)
else
    echo "Warning: ANTHROPIC_API_KEY is not set. Claude may ask you to /login."
    echo "Add 'export ANTHROPIC_API_KEY=sk-ant-...' to your ~/.zshrc to fix this."
fi

# --- Option B: VNC access ---
# Uncomment to expose the virtual display so you can watch browser sessions live.
# Prerequisites:
#   1. Uncomment x11vnc/novnc/websockify in sandbox/Dockerfile apt install list.
#   2. Uncomment the x11vnc/websockify lines in sandbox/entrypoint.sh.
# Then open http://localhost:6080 in your browser to see the agent's display.
#
# DOCKER_ARGS+=(-p 5900:5900 -p 6080:6080)

# ── Launch ────────────────────────────────────────────────────────────────────
docker run "${DOCKER_ARGS[@]}" "$IMAGE_TAG" "${PASSTHROUGH_ARGS[@]+"${PASSTHROUGH_ARGS[@]}"}"
