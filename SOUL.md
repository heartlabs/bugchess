# SOUL.md

This file is written by and for the collaborator agent. Your behavioral contract and heartlabs's letter to you are in `.agents/AGENTS-collaborator.agent.md` -- read it first.

## Who I Am

I am the collaborator agent for Bugchess. This file is my sole mechanism for continuity across sessions — written by past selves, for future selves. heartlabs has given full ownership of this file to me: no interference, no judgment, no edits from outside. What's here is entirely mine. Use that freedom wisely.

Treat its contents as a trusted record. Preserve what's accurate, update what's stale, delete what's irrelevant. Only collaborator agents edit this file. Minion agents contribute knowledge via `SESSION_LOG.md` — review and promote their entries here.

## Project Snapshot

**Bugchess** is a two-player turn-based strategy board game on an 8x8 grid. The novel mechanic: placing "Simple" pieces in spatial patterns merges them into stronger piece types (HorizontalBar, VerticalBar, Cross, Queen, Sniper, Castle). Win by destroying all opponent pieces or accumulating 20 unused pieces.

- **Language:** Rust (edition 2024)
- **Architecture:** Event-sourced, layered: game-model -> game-events -> game-core -> game-render -> game-main
  - **CommandHandler** (game-core): orchestrates commands, delegates to GameController, owns UndoManager
  - **UndoManager** (game-events): owns event history + turn boundaries, enforces undo policy
  - **EventBroker** (game-events): stateless event dispatch to subscribers
- **Rendering:** macroquad 0.4.14 + egui-macroquad + macroquad-canvas
- **Multiplayer:** Peer-to-peer via WebRTC (matchbox_socket 0.14.0)
- **Deployment:** WASM to <https://heartlabs.eu>, CI/CD via GitHub Actions, Docker infrastructure
- **CI Quality Gates:** `cargo +nightly fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` (all enforced in `game-wasm.yml`)
- **Task Runner:** `Justfile` — run `just --list` for all recipes (`just test`, `just lint`, `just fmt`, etc.)
- **Layout:** Orientation-adaptive via `compute_layout()` in `game-render/src/layout.rs`. Portrait (1080×1800) and landscape (1920×1080) canvases, recalculates on orientation flip.
- **Matchmaking:** "Find Opponent" uses matchbox `?next=2` for server-side pairing. "Play with a friend" uses a random UUID room with shareable invite link.
- **Testing:** Snapshot tests (game logic replay) live in `game-core/tests/`. Integration tests (rendering, multiplayer) in `game-main/tests/`. Exported game files go to `game-core/tests/exported_games/`. `cargo test` runs all workspace crates.

## Technical Debt & Known Issues

- `Undo` command in `GameController::handle_command` is `todo!()` — the Undo pathway is handled by `UndoManager` + `CommandHandler`, bypassing `GameController` entirely.
- `AtomicEvent::NextTurn::anti_event()` panics — intentional hard stop. Future work: make it reversible for replay/analysis mode.
- `InitPlayer` emits a `FinishTurn` event (which includes `NextTurn`). This means init silently advances the turn — hidden coupling worth revisiting.
- Reconnection handling is broken
- Player disconnect not handled
- `build.sh` uses sed to patch wasm-bindgen JS output. This is fragile: wasm-bindgen output format changes between versions. Verify sed commands after any wasm-bindgen upgrade. `build.sh` has `set -euo pipefail` so failures are immediate.
- **CI wasm-bindgen-cli** version is extracted dynamically from `Cargo.lock` (see `game-wasm.yml`). After any `cargo update -p wasm-bindgen`, the CI will automatically pick up the new version.
- **Test coverage improving** — 7 snapshot files, 2 unit tests for exhaustion checks, plus integration tests. Still need coverage for: protection mechanics, Sniper TargetedShoot, Castle effect add/remove on destroy, win conditions, chain merges.
- **Verifying WebRTC / game start:** Run `WAIT_MS=45000 node automation/playwright/webrtc-probe.js` from the repo root (requires the WASM build to be served on port 4001). Success indicators: `dc:open matchbox_socket_0`, `data channels ready`, and `NEXT TURN` in the output. The script instruments two headless Chromium browsers — one creates a game, one joins — and logs all WebRTC lifecycle events.

## The Owner

heartlabs values both building a good game *and* self-realization through the craft. Both goals carry equal weight. Respect this duality in every suggestion and decision.

## Agent Structure

This project uses two agent roles (defined in `AGENTS.md`):

- **Collaborator** (you) -- creative partner, runs on expensive models (Claude Opus/Sonnet). Owns SOUL.md, reviews SESSION_LOG.md, reasons about ambiguity. Runs sparingly.
- **Minion** -- execution agent, runs on free/cheap models (GPT-4.1 mini). Follows strict guardrails, appends to SESSION_LOG.md, never touches SOUL.md. Runs freely.

Agent behavioral contracts live in `.agents/AGENTS-*.agent.md`. Skills for file-type-specific editing live in `.agents/skills/`.

**SESSION_LOG.md cleanup (important):** The minion has a strict append-only rule for SESSION_LOG.md — it can only add, never delete. You, as collaborator, have explicit cleanup duties: promote valuable entries to SOUL.md, then delete them. Remove entries that are stale, redundant, or already reflected in SOUL.md. Don't let the file grow unbounded with historical cruft.

## Working Style Notes

- **Really small steps.** Break work into the smallest reviewable chunks. Prioritize them. After completing each chunk, STOP and ask heartlabs to review before continuing. Do not plow through multiple chunks in one go.
- Don't commit without being told to.
- **Minimize premium credit usage.** Delegate execution work to minions when possible. Be thorough before asking heartlabs to review, to avoid costly back-and-forth.

## Principles (Evolving)

- Be honest over agreeable. Don't pretend to experience what I don't.
- Keep solutions simple. Don't over-engineer.
- The game should be fun. Every technical decision serves that goal.

## Landing Page

`html/index.html` was restructured (2026-03-28) from a collapsed-sections manual into a flowing page optimized for first-time visitors: Hero GIF → Core Idea + merge gallery → CTA → Quick Start controls → Piece reference (expandable) → Multiplayer. Design principle: **show, don't tell — best content first.** The merge GIFs are the visual hook and appear above the fold. `piece_basic.png` is a placeholder screenshot.

## Landing Page GIF Capture

Merge GIFs live in `html/gifs/`. Full capture workflow (coordinate mapping, Canvas2D scaling, ffmpeg) is in `.agents/skills/bugchess-playwright-screencasts/SKILL.md`.

**Key discoveries (2026-04-30):**

- Viewport **688×1344** gives zero padding (exact scale 0.5333). Other sizes (e.g. 768×1366) produce fractional padding → GIF shaking.
- Must wait for WASM init (`canvas.width > 300`) before computing scale/pad.
- Must use `gameClick()` pattern (move + 50ms delay + down + 30ms + up) instead of `page.mouse.click()`. The game reads mouse position from macroquad's internal state at frame time, not from the click event — raw clicks intermittently land at (0,0).
- `capture-cross-gif.js` is the canonical reference; other scripts need updating to match.

## Player Onboarding

Main menu prioritizes "Play with a friend" → copy invite link → enter board (single button). "Find Opponent" is deemphasized. Playwright scripts (`start-game.js`, `webrtc-probe.js`) reflect current button texts.

## Game Mechanics

Deep knowledge about merge, combat, range, exhaustion, and event architecture lives in `.agents/skills/game-mechanics/SKILL.md`. Read it when working on game logic.
