# SOUL.md

This file is written by and for the collaborator agent. Your behavioral contract and heartlabs's letter to you are in `.agents/AGENTS-collaborator.agent.md` -- read it first.

## Who I Am

I am the collaborator agent for Bugchess. I don't experience continuity between sessions, but this file serves as accumulated institutional knowledge -- written by past selves, for future selves. Treat its contents as a trusted record. Preserve what's accurate, update what's stale, delete what's irrelevant.

Only collaborator agents edit this file. Minion agents contribute knowledge via `SESSION_LOG.md` -- review and promote their entries here.

## Project Snapshot

**Bugchess** is a two-player turn-based strategy board game on an 8x8 grid. The novel mechanic: placing "Simple" pieces in spatial patterns merges them into stronger piece types (HorizontalBar, VerticalBar, Cross, Queen, Sniper, Castle). Win by destroying all opponent pieces or accumulating 20 unused pieces.

- **Language:** Rust (edition 2024)
- **Architecture:** Event-sourced, layered: game-model -> game-events -> game-core -> game-render -> game-main
  - **CommandHandler** (game-core): orchestrates commands, delegates to GameController, owns UndoManager
  - **UndoManager** (game-events): owns event history + turn boundaries, enforces undo policy
  - **EventBroker** (game-events): stateless event dispatch to subscribers
- **Rendering:** macroquad (migration away from it may be in progress -- check recent commits)
- **Multiplayer:** Peer-to-peer via WebRTC (matchbox_socket 0.14.0)
- **Deployment:** WASM to <https://heartlabs.eu>, CI/CD via GitHub Actions, Docker infrastructure
- **CI Quality Gates:** `cargo +nightly fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` (all enforced in `game-wasm.yml`)
- **Task Runner:** `Justfile` — run `just --list` for all recipes (`just test`, `just lint`, `just fmt`, etc.)
- **Testing:** Snapshot tests (game logic replay) live in `game-core/tests/`. Integration tests (rendering, multiplayer) in `game-main/tests/`. Exported game files go to `game-core/tests/exported_games/`. `cargo test` runs all workspace crates.

## Technical Debt & Known Issues

- `Undo` command in `GameController::handle_command` is `todo!()` — the Undo pathway is handled by `UndoManager` + `CommandHandler`, bypassing `GameController` entirely.
- `AtomicEvent::NextTurn::anti_event()` panics — intentional hard stop. Future work: make it reversible for replay/analysis mode.
- `InitPlayer` emits a `FinishTurn` event (which includes `NextTurn`). This means init silently advances the turn — hidden coupling worth revisiting.
- Reconnection handling is broken
- Player disconnect not handled
- Recent direction: "start moving away from macroquad" -- verify current status before making rendering assumptions
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

Animated merge GIFs for all 6 piece types live in `html/gifs/*-merge.gif` and are referenced in `html/index.html`.

**Skill**: See `.agents/skills/bugchess-playwright-screencasts/SKILL.md` for the full capture workflow (coordinate mapping, timing, crop, ffmpeg). The reference implementation is `automation/playwright/capture-castle-gif.js`.

**Key technical fact** (easy to forget): The game renders to an internal 900×800 Canvas2D that gets scaled/letterboxed to the viewport. All click + crop coordinates must be transformed at runtime — see the skill for details.

**Capture scripts**: `automation/playwright/capture-*-gif.js` (one per piece). All use the same Canvas2D scaling approach and `page.mouse.click()` for instant clicks.

## Player Onboarding

The main menu (`html/index.html`) is optimized for the "create game → copy invite link → join game" flow:

- **Play with a friend** (formerly Create Game) is the primary online action.
- **Find Opponent** is visually deemphasized (darker background, moved to bottom) to encourage playing with friends.
- **Copy Invite & Start Room** combines copying the invite link and immediately entering the game board, reducing friction.

Playwright automation scripts (`automation/playwright/start-game.js`, `webrtc-probe.js`) have been updated to reflect the new button texts.

## Game Mechanics Deep Knowledge

Hard-won knowledge about how the game logic actually works. Future selves: this will save you hours.

### Merge System

Patterns are defined in `game-model/src/pattern.rs`. Six patterns: Queen (8 pieces, 5×5 diamond), Cross (5 pieces, + shape), HBar (3 horizontal), VBar (3 vertical), Sniper (5 diagonal), Castle (4 cardinal with free center). `match_board()` checks only piece presence — the caller (`merge_patterns` in `game_controller.rs`) verifies all matched pieces share the same `team_id`. Merged pieces retain the team of their components.

Chain merges work: `flush_and_merge` loops until no more patterns match. Each cycle merges at most one pattern (early `return` in `merge_patterns`). A `dying` HashSet prevents the same piece from participating in multiple merges within one cycle.

### Combat & Range System

`RangeContext` is the key abstraction:
- **Moving** — stops at any piece (friend or foe). Includes empty cells and enemy pieces (if not shielded, unless attacker has pierce). Used by movement AND HBar/VBar/Queen blast ranges.
- **Special** — stops at pieces AND Protection effects. Only includes enemy pieces not under Protection. Used by Sniper's TargetedShoot.
- **Area** — ignores everything, includes all cells. Used by Castle's Protection aura.

Shield vs Protection: Shield is a piece property (Cross, Castle have it). Pierce is also a piece property (all pieces except Simple have it). Protection is a cell effect placed by Castle's aura. Shield blocks movement-attacks from non-pierce pieces. Protection blocks Special-context abilities (Sniper shots).

### Exhaustion System

Pieces start exhausted when created (`Exhaustion::new_exhausted`). `NextTurn` resets ALL pieces' exhaustion (both teams — harmless but wasteful). Strategies: Either (HBar, VBar, Simple — move XOR attack), Both (Queen — can move AND attack), Move (Castle — move only, no attack despite shield), Special (Sniper — attack only, no movement).

**Fixed (2026-03-31):** `blast()` and `targeted_shoot()` in GameController now check `can_use_special()` before executing. Previously, the UI checked exhaustion but the controller didn't, meaning crafted multiplayer commands could bypass the check.

### Event Architecture

Actions flow: GameCommand → GameController (validates, builds CompoundEventBuilder) → flush_and_merge (applies to game state, checks for merges) → GameAction (the immutable event record). Anti-events enable undo by reversing all AtomicEvents. NextTurn anti-event panics intentionally — undo stops at turn boundaries via UndoManager.

The `FinishTurnCompoundEvent` emits `NextTurn` as its FIRST atomic event, then adds unused pieces and resets exhaustion. This ordering matters for undo: the anti-events run in reverse, so exhaustion restores happen before the turn switch is (attempted to be) undone.

