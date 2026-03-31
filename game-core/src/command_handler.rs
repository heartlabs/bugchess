use log::error;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::{
    game_controller::{GameCommand, GameController},
    game_events::{Event, GameEventObject, PlayerAction},
    multiplayer_connector::MultiplayerConector,
};
use game_events::{event_broker::EventBroker, undo_manager::UndoManager};
use game_model::game::Game;

pub struct CommandHandler {
    past_commands: Arc<Mutex<Vec<GameCommand>>>,
    undo_manager: UndoManager,
    event_broker: EventBroker,
    pub multiplayer_connector: Option<Rc<RefCell<MultiplayerConector>>>,
}

impl CommandHandler {
    pub fn new(event_broker: EventBroker, past_commands: Arc<Mutex<Vec<GameCommand>>>) -> Self {
        CommandHandler {
            past_commands,
            undo_manager: UndoManager::new(),
            event_broker,
            multiplayer_connector: None,
        }
    }

    pub fn handle_new_command(&mut self, game: Game, command: &GameCommand) {
        self.handle_command_internal(game, command);

        if let Some(multiplayer_connector) = self.multiplayer_connector.as_mut() {
            (*multiplayer_connector).borrow_mut().handle_event(command);
        }
    }

    pub fn get_past_commands(&self) -> Vec<GameCommand> {
        self.past_commands
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .to_vec()
    }

    pub fn handle_remote_command(&mut self, game: Game, event_object: &GameEventObject) {
        match &event_object.event {
            Event::PlayerAction(PlayerAction::Connect(_, _)) => {
                let client = self.multiplayer_connector.take().unwrap();
                client.borrow_mut().signal_new_game();
                client.borrow_mut().resend_game_events();
                let _ = self.multiplayer_connector.insert(client);
            }
            Event::GameCommand(game_action) => {
                self.handle_command_internal(game, game_action);
            }
            _ => {}
        }
    }

    fn log_command(&self, command: &GameCommand) {
        if let Ok(mut v) = self.past_commands.lock() {
            v.push(*command);
        } else {
            error!("Could not log command to past_commands")
        }
    }

    fn handle_command_internal(&mut self, game: Game, command: &GameCommand) {
        self.log_command(command);

        if let GameCommand::Undo = command {
            if let Some(anti_event) = self.undo_manager.undo() {
                self.event_broker.dispatch(&anti_event);
            }
        } else {
            let action = GameController::handle_command(game, command)
                .unwrap_or_else(|_| panic!("Could not handle command {:?}", command));

            self.undo_manager.push(action.clone());
            self.event_broker.dispatch(&action);

            if let GameCommand::NextTurn = command {
                self.undo_manager.mark_turn_boundary();
            }
        }
    }
}
