use std::{cell::RefCell, rc::Rc};

use game_events::{
    actions::compound_events::GameAction,
};
use game_model::game::Game;
use crate::event_broker::EventBroker;
use crate::game_controller::{GameCommand, GameController};
use crate::game_events::{Event, EventConsumer, GameEventObject, PlayerAction};
use crate::multiplayer_connector::MultiplayerConector;

pub struct CommandHandler {
    start_of_turn: usize,
    past_events: Vec<GameCommand>,
    pub event_broker: EventBroker,
    pub multiplayer_connector: Option<Rc<RefCell<MultiplayerConector>>>,
}

impl CommandHandler {
    pub fn new(event_broker: EventBroker) -> Self {
        CommandHandler {
            start_of_turn: 0,
            past_events: vec![],
            event_broker,
            multiplayer_connector: None,
        }
    }

    pub fn handle_new_event(&mut self, game: Game, event: &GameCommand) {
        self.past_events.push(event.clone());
        self.handle_event_internal(game, event);
        self.send_event(event);
    }

    pub fn get_past_events(&self) -> &Vec<GameCommand> {
        &self.past_events
    }

    pub fn handle_remote_event(&mut self, game: Game, event_object: &GameEventObject) {
        match &event_object.event {
            Event::PlayerAction(PlayerAction::Connect(_, _)) => {
                let client = self.multiplayer_connector.take().unwrap();
                client.borrow_mut().signal_new_game();
                client.borrow_mut().resend_game_events();
                let _ = self.multiplayer_connector.insert(client);
            }
            Event::GameCommand(game_action) => {
                self.past_events.push(game_action.clone());

                self.handle_event_internal(game, game_action);
            }
            _ => {}
        }
    }

    fn handle_event_internal(&mut self, game: Game, event: &GameCommand) {
        if let GameCommand::Undo = event {
            if self.past_events.len() <= self.start_of_turn {
                return;
            }

            self.event_broker.undo();
        } else {
            let action = GameController::handle_command(game, event).expect(&format!("Could not handle command {:?}", event));

            self.event_broker.handle_new_event(&action);
        }

        if let GameCommand::NextTurn = event {
            self.start_of_turn = self.past_events.len();
        }
    }

    fn send_event(&mut self, event: &GameCommand) {
        if let Some(multiplayer_connector) = self.multiplayer_connector.as_mut() {
            (*multiplayer_connector).borrow_mut().handle_event(event);
        }
    }
}
