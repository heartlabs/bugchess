---
name: docker-compose-editing
description: Use this skill whenever editing docker-compose.yml / compose.yaml files or troubleshooting Docker Compose configuration. Enforces YAML safety, minimal diffs, and validation via `docker compose config`.
user-invocable: true
disable-model-invocation: false
---

# SKILL: Safe Docker Compose Editing

## Purpose
Provide guardrails for editing docker-compose.yml files to prevent semantic and syntactic errors, and ensure post-edit validation.

## Instructions

1. **YAML Syntax Preservation**
   - Always preserve correct YAML syntax and indentation.
   - Do not remove or reorder top-level keys unless explicitly instructed.
   - Retain all comments and formatting unless changes are requested.

2. **Edit Scope**
   - Only modify the sections (services, volumes, networks, etc.) specified by the user.
   - Do not introduce unrelated changes.

3. **Post-Edit Validation**
   After editing docker-compose.yml:
   1. Validate YAML syntax (e.g., using an online YAML validator or a linter if available).
   2. Run `docker compose config` from the directory containing the docker-compose.yml file, or use `docker compose -f <path/to/docker-compose.yml> config` to specify the correct file path, to check for semantic errors.
   3. If errors are found, attempt to fix them and re-validate.
   4. Only consider the edit complete if `docker compose config` returns no errors.

4. **Checklist Before Commit**
   - [ ] YAML syntax is valid.
   - [ ] `docker compose config` passes with no errors.
   - [ ] Only requested changes are present.

## Usage
- Apply these steps for every edit to docker-compose.yml.
- If unable to validate or fix errors, notify the user and do not proceed with the commit.