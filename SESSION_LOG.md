# SESSION_LOG.md

Append-only log for all agent sessions. Each agent's file (`.agents/AGENTS-*.agent.md`) describes when and how to write here.

## 2026-03-31: Claude Opus 4.6 -- Developer Experience Overhaul

- Created `Justfile` with recipes: test, lint, fmt, fmt-check, build, build-wasm, serve, doc
- Added `//!` module-level doc comments to all 5 crate entry points (game-model, game-events, game-core, game-render, game-main)
- Added CI quality gates: `cargo +nightly fmt --check` and `cargo clippy -- -D warnings` to `game-wasm.yml`
- Updated CI actions: `actions/checkout@v2` → `@v4`, replaced unmaintained `raftario/setup-rust-action` with `dtolnay/rust-toolchain@stable`, added `Swatinem/rust-cache@v2`
- Fixed clippy warning: added `Default` impl for `UndoManager`
- Applied `cargo +nightly fmt` to fix 48 pre-existing formatting diffs
- Deleted dead code: `nakama.rs`, `custom_client.rs`, `output.txt`, `game-main/resources/docker/nakama/` directory
- Removed commented-out imports (`//mod nakama`, `//mod custom_client`) and deps (`nakama-rs`, `tokio-tungstenite`, matchbox alternatives) from `game-main/`
- Fixed window title: "Makrochess" → "Bugchess"
- Refreshed `README.md`: fixed 3 broken GIF paths (now point to `html/gifs/`), removed "TODO"/"Coming soon" placeholders, added all 6 merge GIFs in a table layout
