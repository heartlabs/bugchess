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

## 2026-04-15: Minion -- Matchmaking with matchbox `?next=2`

- **Matchmaking support:** Added `?next=2` query parameter to matchbox URL when room_id is "common" (Find Opponent mode). This enables the matchbox server's rudimentary matching service, creating a new room for every 2 players.
- **Refactored URL building:** Extracted `build_url` helper function with unit tests.
- **Preserved existing behavior:** Create Game mode (random UUID room) and explicit room connections remain unchanged, ensuring reconnection still works.
- **Tests added:** Three unit tests verify URL generation for common room, random room, and rooms with special characters.

## 2026-04-18: Claude Opus 4.6 -- UI Optimization for Player Onboarding

- **UI flow optimized:** Reordered main menu to prioritize "Play with a friend" (formerly Create Game) and demote "Find Opponent" (moved to bottom, darker styling).
- **Combined copy and join:** The separate "Copy Invite Link" and "Join Game" buttons are now a single "Copy Invite & Start Room" button that copies the link and immediately transitions the player into the game board.
- **Updated test automation:** Adjusted Playwright test scripts (`start-game.js`, `webrtc-probe.js`) to reflect new button texts and flow.
- **SOUL.md:** Added note about player onboarding improvements.