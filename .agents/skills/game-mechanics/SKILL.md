---
name: game-mechanics
description: Use this skill when working on game logic — merge system, combat, range, exhaustion, or event architecture. Contains hard-won knowledge about how the game engine actually works.
user-invocable: true
disable-model-invocation: false
---

# Skill: Game Mechanics Deep Knowledge

Use this skill when working on game logic — merge system, combat, range, exhaustion, or event architecture. This is hard-won knowledge that saves hours of code archaeology.

## Merge System

Patterns are defined in `game-model/src/pattern.rs`. Six patterns: Queen (8 pieces, 5×5 diamond), Cross (5 pieces, + shape), HBar (3 horizontal), VBar (3 vertical), Sniper (5 diagonal), Castle (4 cardinal with free center). `match_board()` checks only piece presence — the caller (`merge_patterns` in `game_controller.rs`) verifies all matched pieces share the same `team_id`. Merged pieces retain the team of their components.

Chain merges work: `flush_and_merge` loops until no more patterns match. Each cycle merges at most one pattern (early `return` in `merge_patterns`). A `dying` HashSet prevents the same piece from participating in multiple merges within one cycle.

## Combat & Range System

`RangeContext` is the key abstraction:
- **Moving** — stops at any piece (friend or foe). Includes empty cells and enemy pieces (if not shielded, unless attacker has pierce). Used by movement AND HBar/VBar/Queen blast ranges.
- **Special** — stops at pieces AND Protection effects. Only includes enemy pieces not under Protection. Used by Sniper's TargetedShoot.
- **Area** — ignores everything, includes all cells. Used by Castle's Protection aura.

Shield vs Protection: Shield is a piece property (Cross, Castle have it). Pierce is also a piece property (all pieces except Simple have it). Protection is a cell effect placed by Castle's aura. Shield blocks movement-attacks from non-pierce pieces. Protection blocks Special-context abilities (Sniper shots).

## Exhaustion System

Pieces start exhausted when created (`Exhaustion::new_exhausted`). `NextTurn` resets ALL pieces' exhaustion (both teams — harmless but wasteful). Strategies: Either (HBar, VBar, Simple — move XOR attack), Both (Queen — can move AND attack), Move (Castle — move only, no attack despite shield), Special (Sniper — attack only, no movement).

**Fixed (2026-03-31):** `blast()` and `targeted_shoot()` in GameController now check `can_use_special()` before executing. Previously, the UI checked exhaustion but the controller didn't, meaning crafted multiplayer commands could bypass the check.

## Event Architecture

Actions flow: GameCommand → GameController (validates, builds CompoundEventBuilder) → flush_and_merge (applies to game state, checks for merges) → GameAction (the immutable event record). Anti-events enable undo by reversing all AtomicEvents. NextTurn anti-event panics intentionally — undo stops at turn boundaries via UndoManager.

The `FinishTurnCompoundEvent` emits `NextTurn` as its FIRST atomic event, then adds unused pieces and resets exhaustion. This ordering matters for undo: the anti-events run in reverse, so exhaustion restores happen before the turn switch is (attempted to be) undone.
