---
name: markdown-editing
description: Use when editing Markdown (*.md) such as README.md, CHANGELOG.md, AGENTS.md, or docs/*.md. Enforces minimal diffs, preserves headings/lists/code fences/tables, fixes Markdown formatting, and validates via Markdown Preview or markdownlint when available.
user-invocable: true
disable-model-invocation: false
---

# SKILL: Safe Markdown Editing

## Purpose
Safely edit Markdown files with **minimal diffs** while improving clarity and ensuring correct rendering.

## Core Principles
- **Minimal diff:** change only what’s requested + the smallest adjacent formatting fixes needed.
- **Preserve structure:** keep section order and heading hierarchy unless asked.
- **Render correctness:** do not introduce Markdown parsing issues.

## Instructions

1. **Preserve Structure**
   - Keep existing **heading levels** (`#`, `##`, `###`) and section order unless explicitly requested.
   - Preserve **code fences** exactly (``` vs ~~~, language tags, fence length). Never leave a fence unclosed.
   - Preserve **tables** structure; avoid reformatting the whole table unless asked.

2. **Edit Scope**
   - Only modify the content the user requested.
   - If you must touch nearby lines (e.g., to fix a broken list), keep it minimal and note it.

3. **Markdown Formatting Standards**
   - Headings: avoid skipping levels; keep consistent spacing (blank line before/after where appropriate).
   - Lists: keep list marker style consistent within a section; maintain correct indentation for nested lists.
   - Code: keep inline code in single backticks; don’t rewrap commands in a way that changes copy/paste behavior.
   - Links:
     - Prefer **relative links** for repo files.
     - **Do not invent URLs**. If the correct link target is unclear, keep the existing link or add a TODO.
   - Wrapping: do **not** reflow/re-wrap paragraphs unless asked; avoid mass formatting changes.

4. **Clarity and Tone**
   - Maintain the document’s tone (technical vs. product).
   - Prefer concrete, scannable writing: short paragraphs, bullets, and examples where helpful.
   - For READMEs: keep “Quickstart” steps correct and runnable.

5. **Post-Edit Validation**
   - Use **Markdown Preview** if available to confirm:
     - lists render correctly (especially nested lists)
     - code fences render correctly
     - tables render correctly

## Completion Checklist
- [ ] Only requested changes + minimal necessary adjacent fixes.
- [ ] Headings/lists/code fences/tables render correctly in preview.
- [ ] No broken/unclosed code fences.
- [ ] No invented links or large-scale reformatting.