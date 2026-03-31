//! Event system for Bugchess — turns game mutations into replayable, undoable event streams.
//!
//! [`atomic_events::AtomicEvent`]s are the smallest state changes (place, remove, exhaust);
//! [`actions`] compose them into compound [`actions::compound_events::GameAction`]s for moves,
//! attacks, and merges. [`event_broker::EventBroker`] dispatches events to subscribers, and
//! [`undo_manager::UndoManager`] provides per-turn undo by inverting events.
//!
//! Sits above `game-model` and below `game-core` in the architecture.

#![allow(clippy::question_mark)]
pub mod actions;
pub mod atomic_events;
pub mod event_broker;
pub mod undo_manager;
