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

## 2025-03-27: Claude Opus 4.6 — Horizontal Bar GIF Complete + Handoff Prompt

- Built and shipped `html/gifs/horizontal-bar-merge.gif` — approved by heartlabs.
- Abandoned screenshot-per-frame approach (too slow at ~50-100ms/call for 400ms animations).
- Switched to Playwright video recording (`recordVideo` on context) + ffmpeg crop+trim+GIF pipeline.
- Key fix: team 1's setup placement at (5,5) swooshes across crop area; wait 4000ms after canvas visible.
- Key fix: `getByText('Offline')` resolves to 2 elements; use `a[onclick="runOffline()"]` instead.
- Cleaned up stale scripts: removed `capture-pieces-auto.js`, `capture-pieces.js`, debug artifacts.
- Updated SOUL.md with full GIF capture pipeline knowledge.

### HANDOFF PROMPT — Create Merge GIFs for Remaining 5 Pieces

**Goal:** Create animated GIFs for VerticalBar, Cross, Castle, Sniper, and Queen, matching the approved `html/gifs/horizontal-bar-merge.gif` in style and quality.

**Reference implementation:** `automation/playwright/capture-horizontal-bar-gif.js` — study this file end-to-end. It is the proven template. Copy it per piece and modify click plan, crop coords, and output path.

**Read first:** The "Landing Page GIF Capture Pipeline" section of SOUL.md has all technical details (board geometry, crop formulas, timing constants, critical lessons).

---

#### Setup Context

`set_up_pieces` in `game-main/src/states/loading.rs`:
- `InitPlayer(6)` × 2 teams
- `PlacePiece(2,2)` + NextTurn for team 0
- `PlacePiece(5,5)` + NextTurn for team 1

After setup: team 0 has **5 unused pieces**, 1 placed at (2,2). Team 1 has 5 unused + 1 at (5,5). It's team 0's turn in Place state.

**Multiple placements per turn:** You can place multiple pieces on the same turn as long as you have unused pieces — no need to click End Turn between placements (unless you need MORE unused pieces than you start with).

**Accumulating more pieces:** Click End Turn (canvas coords ~x=780, y=40) twice to do one round trip: your end turn → opponent's turn (they do nothing in offline mode) → your turn again with +1 unused piece. **WARNING: game ends at 20 unused pieces.** Team 0 starts with 5, so max ~14 round trips before game over.

---

#### Piece-by-Piece Click Plans

All patterns verified against `game-model/src/pattern.rs`. Board coordinates are (x, y) where x=column, y=row.

##### 1. VerticalBar — 2 placements, no End Turn needed

Pattern (3×3 starting at grid (1,1)):
```
Free Own  Free       abs: (1,1)=Free (2,1)=Own  (3,1)=Free
Free Own  Free       abs: (1,2)=Free (2,2)=Own✓ (3,2)=Free
Free Own  Free       abs: (1,3)=Free (2,3)=Own  (3,3)=Free
```
Pieces to place: **(2,1)** and **(2,3)**. Initial piece at (2,2) is center ✓.
Crop: 3×3 centered on (2,2) — **identical crop to horizontal bar**.
Output: `html/gifs/vertical-bar-merge.gif`

##### 2. Cross — 4 placements, no End Turn needed (5 unused - 4 = 1 remaining)

Pattern (3×3 starting at grid (1,1)):
```
Any  Own  Any        abs: (2,1)=Own
Own  Own  Own        abs: (1,2)=Own  (2,2)=Own✓  (3,2)=Own
Any  Own  Any        abs: (2,3)=Own
```
Pieces to place: **(2,1)**, **(1,2)**, **(3,2)**, **(2,3)**. Order matters — last placement triggers merge.
Crop: 3×3 centered on (2,2) — **identical crop to horizontal bar**.
Output: `html/gifs/cross-merge.gif`

##### 3. Sniper — 4 placements, no End Turn needed

Pattern (3×3 starting at grid (1,1)):
```
Own  Any  Own        abs: (1,1)=Own              (3,1)=Own
Any  Own  Any        abs:         (2,2)=Own✓
Own  Any  Own        abs: (1,3)=Own              (3,3)=Own
```
Pieces to place: **(1,1)**, **(3,1)**, **(1,3)**, **(3,3)**. Last triggers merge.
Crop: 3×3 centered on (2,2) — **identical crop to horizontal bar**.
Output: `html/gifs/sniper-merge.gif`

##### 4. Castle — 3 placements, no End Turn needed

**Tricky:** Castle center must be **Free** — the initial piece at (2,2) cannot be the center.
Solution: offset the pattern so (2,2) is an OwnPiece, not the center.

Pattern (3×3 starting at grid **(1,2)**):
```
Any  Own  Any        abs: (2,2)=Own✓
Own  Free Own        abs: (1,3)=Own  (2,3)=Free  (3,3)=Own
Any  Own  Any        abs: (2,4)=Own
```
Pieces to place: **(1,3)**, **(3,3)**, **(2,4)**. Last triggers merge.
Merged Castle appears at: pattern start + (1,1) = (2,3).
Crop: 3×3 centered on **(2,3)** — different from horizontal bar!
```javascript
const cropX = Math.round(SHIFT_X + 1 * CELL + CROP_ADJUST_X);  // originCellX = 1
const cropY = Math.round(SHIFT_Y + 2 * CELL + CROP_ADJUST_Y);  // originCellY = 2
```
Output: `html/gifs/castle-merge.gif`

##### 5. Queen — 7 placements, needs 2 End Turn round trips first

**Complex:** 5×5 pattern, 8 OwnPiece positions, center must be Free.
Solution: offset pattern to start at grid **(0,2)** so initial piece at (2,2) maps to pattern (2,0) = OwnPiece.

Pattern (5×5 starting at grid (0,2)):
```
Any  Any  Own  Any  Any     abs: (2,2)=Own✓
Any  Own  Free Own  Any     abs: (1,3)=Own  (2,3)=Free  (3,3)=Own
Own  Free Free Free Own     abs: (0,4)=Own  (1,4)=Free (2,4)=Free (3,4)=Free  (4,4)=Own
Any  Own  Free Own  Any     abs: (1,5)=Own  (2,5)=Free  (3,5)=Own
Any  Any  Own  Any  Any     abs: (2,6)=Own
```
Pieces to place (7): **(1,3)**, **(3,3)**, **(0,4)**, **(4,4)**, **(1,5)**, **(3,5)**, **(2,6)**.
Merged Queen appears at: pattern start + (2,2) = (2,4).
Start with 5 unused; need 7 → **4 End Turn clicks** (2 round trips) before recording trim starts.

**End Turn accumulation sequence (before trim-start):**
```javascript
await delay(SETUP_SETTLE_MS);
// Accumulate 2 more pieces: 2 round trips × 2 clicks each
for (let i = 0; i < 2; i++) {
  await canvas.click({ position: { x: 780, y: 40 } }); // end our turn
  await page.mouse.move(5, 5);
  await delay(1500); // wait for opponent turn to complete
  await canvas.click({ position: { x: 780, y: 40 } }); // end opponent's turn
  await page.mouse.move(5, 5);
  await delay(1500); // wait for our turn to resume
}
await delay(1000); // extra settle time
// NOW mark trim-start and begin placing
```

Crop: **5×5** centered on (2,4):
```javascript
const N = 5;
const centerX = 2, centerY = 4;
const originCellX = centerX - Math.floor(N / 2);  // = 0
const originCellY = centerY - Math.floor(N / 2);  // = 2
const cropX = Math.round(SHIFT_X + originCellX * CELL + CROP_ADJUST_X);
const cropY = Math.round(SHIFT_Y + originCellY * CELL + CROP_ADJUST_Y);
const cropW = Math.round(CELL * N + CROP_ADJUST_W);
const cropH = Math.round(CELL * N + CROP_ADJUST_H);
```
GIF size for Queen: scale to ~445×445 (preserving same cell visual size as 3×3 at 267), or use a different size — your call.
Output: `html/gifs/queen-merge.gif`

---

#### Technical Checklist Per Piece

1. Copy `capture-horizontal-bar-gif.js` → `capture-<piece>-gif.js`
2. Update: OUTPUT_GIF path, click coordinates, crop coordinates, GIF_SIZE (if 5×5), timing
3. Build: `bash build.sh`
4. Serve: `basic-http-server html/ --addr 0.0.0.0:4000` (background)
5. Run: `node automation/playwright/capture-<piece>-gif.js`
6. **Visually inspect** the GIF — automated metrics are unreliable
7. Verify: correct cells visible, no hover effects, no black frames, merge animation smooth
8. Clean up `html/gifs/video/` after each run

#### Critical Lessons (from painful debugging)

- **Always move mouse to (5,5) canvas coords after every click** — prevents green hover/move lines
- **NEVER trust automated frame-diff verification** — always open the GIF and watch it
- **page.screenshot() is too slow for animations** — use video recording only
- **Team 1's setup piece swooshes across the board** — the 4000ms settle time is mandatory
- **CROP_ADJUST values are empirical** — if crop looks wrong, take a screenshot and measure pixel positions
- **For pieces beyond 3×3, crop adjustments may need recalibration** — verify 5×5 crops carefully
- **Castle and Queen have offset patterns** — the crop center is NOT (2,2) for these pieces

## 2026-03-27: Claude Opus 4.6 — Castle GIF + Canvas2D Scaling Fix

- Built and shipped `html/gifs/castle-merge.gif`.
- **Discovered critical Canvas2D scaling bug**: game renders to 900×800 internal canvas, which macroquad_canvas scales+letterboxes to viewport (1280×922). Old `cellCenter()` used game-internal coords as raw viewport click positions → clicks landed ~121px too far left (~1 cell offset). The horizontal bar GIF "worked by accident" because the shifted clicks still formed a valid pattern.
- **Fix**: compute `scale = min(viewportW/900, viewportH/800)` and `leftPad = (viewportW - 900×scale)/2` from `canvas.boundingBox()` at runtime. Transform all clicks + crop: `viewport_px = pad + game_px × scale`.
- Castle merge animation needs ~3.5s after final placement (not 2.5s like horizontal bar); 4 pieces merging takes longer than 3.
- Updated `capture-castle-gif.js` as the new reference implementation (dynamic scaling). Old `capture-horizontal-bar-gif.js` still uses hardcoded CROP_ADJUST values.
- Updated SOUL.md: added Canvas2D Scaling section, updated completed GIFs, noted horizontal bar may need recapture.

## 2026-03-27: Claude Opus 4.6 — All Remaining Merge GIFs Created

- Created GIF capture scripts and produced GIFs for all remaining pieces:
  - `vertical-bar-merge.gif` — 267×267, 125 frames, 2 placements at (2,1), (2,3)
  - `cross-merge.gif` — 267×267, 135 frames, 4 placements at (2,1), (1,2), (3,2), (2,3)
  - `sniper-merge.gif` — 267×267, 135 frames, 4 placements at (1,1), (3,1), (1,3), (3,3)
  - `queen-merge.gif` — 445×445, 163 frames, 7 placements + 2 End Turn round trips for extra pieces
- All scripts use the corrected Canvas2D scaling approach from castle script (dynamic scale/padding from boundingBox).
- Queen required End Turn accumulation to get 7 unused pieces (starts with 5). Pattern anchored at (0,2), crop 5×5 centered on (2,4).
- All GIFs pending visual approval from heartlabs.
- Updated SOUL.md with complete GIF status.

## 2026-03-27: Claude Opus 4.6 — Timing Fix, index.htm Update, Screencast Skill

- **Timing fix**: Reduced inter-click delay from 200ms to 50ms (placements are the "boring" part); increased post-merge wait to 5000ms (3×3) / 6000ms (Queen) so the merged piece is fully visible; kept 1200ms final hold. Re-captured all 5 GIFs (castle, vertical bar, cross, sniper, queen).
- **index.htm**: Updated `<img src>` attributes for Cross, Bar, Queen, Sniper, and Castle to reference the actual `*-merge.gif` files.
- **New skill**: Created `.agents/skills/bugchess-playwright-screencasts/SKILL.md` — documents the full capture workflow (coordinate mapping, timing, cropping, ffmpeg conversion). Flexible and not hardcoded to specific piece patterns.
- **SOUL.md slimmed**: Replaced the ~90-line GIF Pipeline section with a compact 8-line pointer to the skill + key technical fact (Canvas2D scaling). Went from detailed implementation notes to "here's where to find it".
