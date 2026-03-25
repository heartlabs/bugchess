# SESSION_LOG.md

Append-only log for all agent sessions. Each agent's file (`.agents/AGENTS-*.agent.md`) describes when and how to write here.

## 2026-03-06: Unknown Model -- Dependency Updates & UI/Menu Fix

- Updated WASM-related dependencies (`wasm-bindgen`, `wasm-bindgen-futures`, `web-sys`) in `game-main/Cargo.toml` to latest versions.
- Ran `cargo check` and `cargo test --workspace`: all tests passed, only warnings (no errors).
- Fixed start menu layout and font scaling: menu is now readable and usable again.
- Root cause was an accidental font scale of 10.0 in egui_setup_fonts; set to 1.0.
- Centered menu, added spacing, and set explicit font sizes for clarity.

## 2026-03-06: Unknown Model -- WASM Browser Loading Fix

- After wasm-bindgen was updated to 0.2.114, the game stopped loading in the browser.
- Three root causes were identified and fixed:
  1. `build.sh` sed patches outdated: wasm-bindgen 0.2.114 changed its JS output format. Updated all sed commands to match.
  2. `index.htm` plugin registration used wrong key: fixed `register_plugin` to set `importObject["./bugchess_bg.js"]`.
  3. WebGL version mismatch: miniquad 0.4.8 defaults to WebGL1 but macroquad 0.4.14 uses WebGL2 functions. Added `webgl_version: WebGLVersion::WebGL2` to window config.
- Also updated gl.js to match miniquad master (version 2).

## 2026-03-07: Claude Opus 4.6 -- Memory System Setup

- Introduced dual-file memory system: SOUL.md (curated, whitelist-only) + SESSION_LOG.md (append-only, all models).
- Rewrote AGENTS.md with model whitelist, clear rules for both whitelisted and non-whitelisted models.
- Cleaned up SOUL.md: moved session-specific work logs here, promoted the wasm-bindgen sed fragility lesson into SOUL.md's Technical Debt section.
- Migrated pre-existing session logs from SOUL.md attributed as "Unknown Model" since the originating model is not known.


## 2026-03-07: GitHub Copilot -- Skills Directory Strategy

- Moved all agent skill and guardrail files into a dedicated `skills/` folder for clarity, discoverability, and scalability.
- Standardized skill file naming to lowercase with hyphens and `.skill.md` suffix (e.g., `docker-compose-editing.skill.md`).
- Updated `AGENTS.md` to instruct all models to check the `skills/` folder before performing specialized or sensitive edits.
- Rationale: Centralizing skills prevents clutter in the project root, makes it easy for future models to find, update, and add new skills, and supports project growth.
- Added explicit reference in `AGENTS.md` to `skills/docker-compose-editing.skill.md` for Docker Compose edits, ensuring all models follow best practices and post-edit validation.
- All skill files should be written to be model-agnostic and usable by as many different models as possible. Avoid model-specific instructions or dependencies; provide clear, general-purpose guardrails and workflows. This maximizes the benefit of accumulated project knowledge and ensures consistent behavior regardless of which agent or model is active.

## 2026-07-14: GitHub Copilot (Claude Sonnet 4.6) -- matchbox_socket 0.6.1 → 0.14.0 Upgrade

- Upgraded `matchbox_socket` from `0.6.1` to `0.14.0` in `game-main/Cargo.toml`.
- Four API changes were required in `game-main/src/matchbox.rs`:
  1. **receive**: `socket.receive()` → `socket.channel_mut(0).receive()`
  2. **send**: `socket.send(data, peer)` → `socket.channel_mut(0).send(data, peer)`
  3. **update_peers filter**: `update_peers()` now returns both `Connected` and `Disconnected` events; added `.filter(|(_, state)| *state == PeerState::Connected)` to avoid spurious disconnects being treated as new connections.
  4. **socket.id() caching**: `socket.id()` now takes `&mut self`, but the `own_player_id()` trait method takes `&self`. Solved by adding `own_id: Option<String>` to `MatchboxClient`, refreshed in `accept_new_connections()`.
- `add_reliable_channel()` builder method and `PeerId` struct are unchanged through 0.14.0.
- `cargo check` passes (3 pre-existing warnings only). `bash build.sh` succeeded in ~70s.
- Verified end-to-end with `webrtc-probe.js`: both browsers connect, ICE negotiates, data channels open, `NEXT TURN` logs confirm game actually starts.

## 2026-03-07: Claude Opus 4.6 -- Agent Structure Refactoring

- Refactored agent management from a single AGENTS.md with branching rules into a clean two-agent architecture:
  - `AGENTS.md` → slim project context (~35 lines), loaded by all agents
  - `.agents/AGENTS-collaborator.agent.md` → new file for creative/expensive models, contains heartlabs's original letter + behavioral contract
  - `.agents/AGENTS-minion.agent.md` → updated with SESSION_LOG duty and SOUL.md prohibition
- Key design principle: minion gets everything delivered in its agent file (self-contained); collaborator is trusted to seek out SOUL.md and SESSION_LOG.md on its own.
- Renamed "co-maintainer" → "collaborator" at heartlabs's request.
- Preserved heartlabs's original personal letter (Way of Working, Philosophy) verbatim in the collaborator agent file rather than paraphrasing it.
- Updated SOUL.md with new Agent Structure section documenting the two-role system.
- Fixed stale cross-references in SOUL.md and SESSION_LOG.md headers.

## 2026-03-25: GitHub Copilot (GPT-5.3-Codex) -- Landing Piece GIFs Completed

- Replaced stale `create-gifs.sh` paths and placeholder behavior with a sprite-based generator that outputs all required landing GIFs into `html/gifs/`.
- Generator now uses in-game texture atlas `game-render/resources/sprites/insekten4.png` and piece crop coordinates aligned with `SpriteRender::piece_sprite_rect`.
- Added built-in validation in script (`ffprobe`) to fail if any GIF has 1 or fewer frames.
- Updated `create-gifs.js` to delegate to `create-gifs.sh` so both entrypoints run the same validated pipeline.
- Verified outputs: `piece_basic.gif`, `piece_cross.gif`, `piece_bar.gif`, `piece_queen.gif`, `piece_sniper.gif`, `piece_castle.gif` all generated and animated (26 frames each).

## 2026-03-25: GitHub Copilot (GPT-5.3-Codex) -- Script Move + New Playwright Screenshot Skill

- Moved GIF helper scripts from repo root into `automation/` (`automation/create-gifs.sh` and `automation/create-gifs.js`).
- Updated moved shell script path resolution to target repo root from inside `automation/`.
- Deleted old root-level script files (`create-gifs.sh`, `create-gifs.js`) as cleanup.
- Added new skill: `.agents/skills/bugchess-playwright-screenshots/SKILL.md` documenting build/serve/playwright-open/place-piece/screenshot workflow.
- Validated moved script entrypoint by running `node automation/create-gifs.js` successfully.

## 2026-03-25: GitHub Copilot (GPT-5.3-Codex) -- End-of-Day Handoff (GIF work paused)

- User removed generated images/GIFs because current capture quality did not yet meet requirements.
- Kept automation changes that matter for next session: `automation/playwright/capture-pieces-auto.js` updates, moved GIF scripts under `automation/`, and Playwright screenshot skill scaffold in `.agents/skills/bugchess-playwright-screenshots/`.
- Next session should resume from screenshot-first validation of board-only framing and reliable `(2,2)`-anchored merge choreography before re-attempting final GIF assets.
