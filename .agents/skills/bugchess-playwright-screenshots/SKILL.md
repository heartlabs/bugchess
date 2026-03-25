---
name: bugchess-playwright-screenshots
description: Use this skill to build and serve Bugchess locally, open it with Playwright, interact until pieces can be placed on the board, and capture screenshots for further workflows.
user-invocable: true
disable-model-invocation: false
---

# SKILL: Bugchess Local Playwright Screenshot Workflow

## Purpose
Provide a repeatable workflow to run Bugchess locally and reach a reliable Playwright state where you can place pieces on the board and capture screenshots.

## Instructions

1. **Build the Web Game**
   - From repository root, run:
     - `bash build.sh`
   - Ensure build completes without errors (warnings are acceptable unless the task requires fixing them).

2. **Serve `html/` Locally**
   - Start local server from repository root:
     - `basic-http-server html/ --addr 0.0.0.0:4000`
   - Keep the server running while Playwright scripts execute.

3. **Prepare Playwright Runtime**
   - Install automation dependencies:
     - `npm --prefix automation/playwright install`
   - Use base URL:
     - `BASE_URL=http://127.0.0.1:4000/index.htm`

4. **Open Game and Enter Playable Board State**
   - Launch a Playwright script that opens the page and clicks **Offline**.
   - Wait for `#glcanvas` to become visible.
   - Confirm board is interactive by clicking a valid board cell and observing that piece placement starts to occur.

5. **Optional: Capture Screenshots**
   Only do this step if the task requires screenshots for documentation, bug reports, or further processing:
   - Capture full-canvas or clipped board screenshots with Playwright:
     - Use `page.screenshot({ path, clip })` for focused board captures.
   - Store outputs under a deterministic folder in the project and inform the user where you stored them.

6. **Interaction Scope for This Skill**
   - This skill guarantees setup up to: build → serve → open via Playwright → reach piece placement on board → screenshot capture.
   - Advanced gameplay logic and choreography is intentionally out of scope and should be solved per-task.

## Validation Checklist
- [ ] `bash build.sh` succeeds.
- [ ] Local server is running on `http://127.0.0.1:4000`.
- [ ] Playwright opens game and reaches visible canvas (`#glcanvas`).
- [ ] Expected board interaction is performed.
- [ ] If applicable: Expected screenshot files are created in target output folder.
