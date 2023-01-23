use std::{cell::RefCell, rc::Rc};

use crate::{
    game_controller::{GameCommand, GameController},
    game_events::{Event, GameEventObject, PlayerAction},
    multiplayer_connector::MultiplayerConector,
};
use game_events::event_broker::EventBroker;
use game_model::game::Game;

pub struct CommandHandler {
    start_of_turn: usize,
    past_commands: Vec<GameCommand>,
    event_broker: EventBroker,
    pub multiplayer_connector: Option<Rc<RefCell<MultiplayerConector>>>,
}

impl CommandHandler {
    pub fn new(event_broker: EventBroker) -> Self {
        CommandHandler {
            start_of_turn: 0,
            past_commands: vec![],
            event_broker,
            multiplayer_connector: None,
        }
    }

    pub fn handle_new_command(&mut self, game: Game, command: &GameCommand) {
        self.past_commands.push(command.clone());
        self.handle_command_internal(game, command);
        self.send_command(command);
    }

    pub fn get_past_commands(&self) -> &Vec<GameCommand> {
        &self.past_commands
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
                self.past_commands.push(game_action.clone());

                self.handle_command_internal(game, game_action);
            }
            _ => {}
        }
    }

    fn handle_command_internal(&mut self, game: Game, command: &GameCommand) {
        if let GameCommand::Undo = command {
            if self.past_commands.len() <= self.start_of_turn {
                return;
            }

            self.event_broker.undo();
        } else {
            let action = GameController::handle_command(game, command)
                .expect(&format!("Could not handle command {:?}", command));

            self.event_broker.handle_new_event(&action);
        }

        if let GameCommand::NextTurn = command {
            self.start_of_turn = self.past_commands.len();
        }
    }

    fn send_command(&mut self, command: &GameCommand) {
        if let Some(multiplayer_connector) = self.multiplayer_connector.as_mut() {
            (*multiplayer_connector).borrow_mut().handle_event(command);
        }
    }
}
