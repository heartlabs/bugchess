# SESSION_LOG.md

Append-only log for all agent sessions. Each agent's file (`.agents/AGENTS-*.agent.md`) describes when and how to write here.

## 2026-03-31: Claude Opus 4.6 -- Code Review & Bug Fix

- **Bug fixed:** `blast()` and `targeted_shoot()` in `GameController` lacked exhaustion checks. The UI prevented exploits but multiplayer remote commands could bypass it. Added `can_use_special()` guards.
- **Dead code removed:** Unused `exhaustion_clone` variable in `targeted_shoot()`.
- **Tests added:** 2 snapshot tests (0006: HBar merge + blast, 0007: Cross merge + capture) and 2 unit tests verifying exhaustion rejection for blast/shoot.
- **SOUL.md:** Added "Game Mechanics Deep Knowledge" section documenting merge system, combat/range system, exhaustion system, and event architecture.

## 2026-03-31: Claude Opus 4.6 -- Developer Experience Overhaul

- Created `Justfile` with recipes: test, lint, fmt, fmt-check, build, build-wasm, serve, doc
- Added `//!` module-level doc comments to all 5 crate entry points
- Added CI quality gates: `cargo +nightly fmt --check` and `cargo clippy -- -D warnings` to `game-wasm.yml`
- Updated CI actions to current versions, added `Swatinem/rust-cache@v2`
- Fixed clippy warning: added `Default` impl for `UndoManager`
- Deleted dead code: `nakama.rs`, `custom_client.rs`, `output.txt`, nakama docker config
- Fixed window title: "Makrochess" → "Bugchess"
- Refreshed `README.md`: fixed broken GIF paths, added all 6 merge GIFs

