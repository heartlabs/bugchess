use game_events::{
    actions::compound_events::GameAction,
    game_events::{Event, GameEventObject, PlayerAction},
};

use indexmap::IndexMap;
use miniquad::{debug, info};

use std::collections::HashMap;

pub trait MultiplayerClient {
    fn is_ready(&self) -> bool;
    fn accept_new_connections(&mut self) -> Vec<String>;
    fn recieved_events(&mut self) -> Vec<GameEventObject>;
    fn send(&mut self, game_object: &GameEventObject, opponent_id: &str);
    fn own_player_id(&self) -> Option<String>;
}

pub struct MultiplayerConector {
    registered_events: IndexMap<String, GameEventObject>,
    client: Box<dyn MultiplayerClient>,
    pub opponent_id: Option<String>,
    pub override_own_player_index: Option<usize>,
}

impl MultiplayerConector {
    pub fn new(client: Box<dyn MultiplayerClient>) -> Self {
        let connector = MultiplayerConector {
            registered_events: IndexMap::new(),
            client,
            opponent_id: None,
            override_own_player_index: None,
        };

        connector
    }

    pub fn is_ready(&self) -> bool {
        self.client.is_ready()
            && self.client.own_player_id().is_some()
            && self.opponent_id.is_some()
    }

    pub fn matchmaking(&mut self) {
        for peer in self.client.accept_new_connections() {
            info!("Found a peer {:?}", peer);
            self.opponent_id = Some(peer);
        }
    }

    pub fn get_own_player_index(&self) -> Option<usize> {
        if self.override_own_player_index.is_some() {
            return self.override_own_player_index;
        }

        if let Some(opponent_id) = self.opponent_id.as_ref() {
            if let Some(own_player_id) = &self.client.own_player_id() {
                if opponent_id < own_player_id {
                    return Some(1);
                }

                return Some(0);
            }
        }

        return None;
    }

    pub fn get_own_player_id(&self) -> Option<String> {
        self.client.own_player_id()
    }

    pub fn try_recieve(&mut self) -> Vec<GameEventObject> {
        let mut events = vec![];
        for event_object in self.client.recieved_events() {
            if self.register_event(&event_object) {
                debug!("Received event: {}", &event_object);

                if let Event::PlayerAction(PlayerAction::Connect(name, index)) = &event_object.event
                {
                    debug!("Player {} connected with supposed index {}.", name, index);

                    self.opponent_id = Some(name.clone());

                    events.push(event_object);
                } else {
                    events.push(event_object);
                }
            } else {
                debug!("Event already received before: {}", event_object);
            }
        }

        events
    }

    fn register_event(&mut self, event_object: &GameEventObject) -> bool {
        self.registered_events
            .insert(event_object.id.clone(), event_object.clone())
            .is_none()
    }

    pub fn handle_event(&mut self, game_action: &GameAction) {
        let sender = self.get_own_player_id().expect("Own player ID unknown");

        let event = &GameEventObject::new(Event::GameAction(game_action.clone()), &sender);

        self.send(event);
    }

    fn send(&mut self, event: &GameEventObject) {
        let opponent_id = self.opponent_id.as_ref().unwrap().clone();
        self.register_event(event);
        self.client.send(event, &opponent_id);
        debug!("Sent event: {}", event);
    }

    pub fn signal_connect(&mut self) {
        let game_object = &GameEventObject::new(
            Event::PlayerAction(PlayerAction::Connect(
                self.get_own_player_id().unwrap().to_string(),
                self.get_own_player_index().unwrap(),
            )),
            &self.get_own_player_id().unwrap(),
        );

        self.send(game_object);
    }

    pub fn signal_new_game(&mut self) {
        let own_player_id = self.get_own_player_id().unwrap();
        let opponent_id = self.opponent_id.as_ref().unwrap().clone();

        let player_order = if self.get_own_player_index().unwrap() == 0 {
            (own_player_id.clone(), opponent_id)
        } else {
            (opponent_id, own_player_id.clone())
        };
        let game_object = &GameEventObject::new(
            Event::PlayerAction(
                PlayerAction::NewGame(player_order), // you can only signal a new game if you are the first
            ),
            &own_player_id,
        );

        self.send(game_object);
    }

    pub fn resend_game_events(&mut self) {
        let registered_events: Vec<GameEventObject> = self
            .registered_events
            .values()
            .map(|e| e.clone())
            .filter(|e| matches!(e.event, Event::GameAction(_)))
            .collect();

        registered_events.iter().for_each(|e| self.send(e));
    }
}
