FROM ubuntu:24.04

# Avoid interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

# ── Layer 1: System packages ──────────────────────────────────────────────────
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libasound2-dev \
    curl \
    git \
    wget \
    sed \
    xvfb \
    ffmpeg \
    ca-certificates \
    gnupg \
    # --- Option B: VNC access ---
    # Uncomment to support exposing the virtual display over VNC + noVNC.
    # Also uncomment the x11vnc/websockify lines in entrypoint.sh and the
    # port mappings in claude.sh.
    # x11vnc \
    # novnc \
    # websockify \
    && rm -rf /var/lib/apt/lists/*

# ── Layer 2: Node.js (LTS) + Claude Code + Pi ─────────────────────────────────────
RUN curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

RUN npm install -g @anthropic-ai/claude-code

# ── Layer 3: Playwright + Chromium ───────────────────────────────────────────
# Installed globally so all Playwright scripts in the repo can use it.
RUN npm install -g playwright \
    && npx playwright install chromium \
    && npx playwright install-deps chromium

# ── Layer 4: Rust toolchain ───────────────────────────────────────────────────
ENV RUSTUP_HOME=/root/.rustup
ENV CARGO_HOME=/root/.cargo
ENV PATH=/root/.cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --no-modify-path --default-toolchain stable \
    && rustup toolchain install nightly \
    && rustup target add wasm32-unknown-unknown

# ── Layer 5: Cargo tools ──────────────────────────────────────────────────────
# wasm-bindgen-cli must match the version pinned in Cargo.lock.
# claude.sh reads the version at runtime and passes it here as a build arg,
# so the image tag includes the version (bugchess-sandbox:wbg-X.Y.Z).
# A new image is only built when the version changes.
ARG WASM_BINDGEN_VERSION
RUN cargo install just basic-http-server \
    && cargo install wasm-bindgen-cli --version "$WASM_BINDGEN_VERSION"

# -- Pi
# RUN curl -fsSL "https://raw.githubusercontent.com/Dicklesworthstone/pi_agent_rust/main/install.sh?$(date +%s)" \
#     | bash -s -- --yes --easy-mode
# ENV PATH="$HOME/.local/bin:$PATH"
RUN npm install -g @mariozechner/pi-coding-agent

# ── Workspace ─────────────────────────────────────────────────────────────────
WORKDIR /workspace

COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
