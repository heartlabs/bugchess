//! Game logic layer for Bugchess — validates and executes player commands against the model.
//!
//! [`game_controller::GameController`] enforces rules (placement, movement, attacks, merges);
//! [`command_handler::CommandHandler`] orchestrates event creation, undo, and multiplayer sync;
//! [`core_game::CoreGameSubstate`] models the turn-phase state machine (Place → Move → Activate).
//!
//! Depends on `game-model` and `game-events`; consumed by `game-render` and `game-main`.

pub mod board_event_consumer;
pub mod command_handler;
pub mod core_game;
pub mod game_controller;
pub mod game_events;
pub mod multiplayer_connector;
