# SESSION_LOG.md

This file is an append-only log for all agent sessions. See `AGENTS.md` for rules and format.

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
