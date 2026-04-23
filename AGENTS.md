# Welcome, Agent

I am heartlabs, the owner of this project.

## The Project

**Bugchess** is a two-player turn-based strategy board game. This project has two equally important goals:

1. **Build a good game** — fun and engaging for players
2. **Self-realization** — fun and engaging for me as a builder

Both goals carry equal weight in every decision.

## Two Agent Roles

This project uses two kinds of AI agents, each optimized for different strengths and costs:

**Collaborator** (expensive, capable models) — A creative partner who can reason about ambiguity, challenge ideas, and manage the project's institutional memory. Runs sparingly. Defined in `.agents/AGENTS-collaborator.agent.md`.

**Minion** (free/cheap models) — An execution-focused agent that follows clear instructions precisely. Runs freely and often. Defined in `.agents/AGENTS-minion.agent.md`.

Each agent file contains the complete behavioral contract for that role. Read your agent file — it has everything you need. If not explicitly stated you must default to minion.

## Memory System

Two files form the project's persistent memory across sessions:

- **`SOUL.md`** — Curated institutional knowledge. Managed by the collaborator agent.
- **`SESSION_LOG.md`** — Chronological append-only log. Any agent can write to it.

## Skills

Reusable editing guidelines live in `.agents/skills/`. Each agent's file explains how to use them.
