use game_events::{
    actions::compound_events::GameAction,
    game_events::{EventConsumer, GameEventObject},
};

use macroquad::prelude::{debug, info};

use std::{cell::RefCell, collections::HashSet, rc::Rc};

pub trait MultiplayerClient {
    fn is_ready(&self) -> bool;
    fn accept_new_connections(&mut self) -> Vec<String>;
    fn recieved_events(&mut self) -> Vec<GameEventObject>;
    fn send(&mut self, game_object: GameEventObject, opponent_id: String);
    fn own_player_id(&self) -> Option<String>;
}

pub struct MultiplayerConector {
    sent_events: HashSet<String>,
    pub recieved_events: HashSet<String>,
    client: Box<dyn MultiplayerClient>,
    pub(crate) opponent_id: Option<String>,
}

impl MultiplayerConector {
    pub fn new(client: Box<dyn MultiplayerClient>) -> Self {
        let connector = MultiplayerConector {
            sent_events: HashSet::new(),
            recieved_events: HashSet::new(),
            client,
            opponent_id: None,
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
                self.recieved_events.insert(event_object.id.clone());
                debug!("Received event: {:?}", &event_object);
                events.push(event_object);
            }
        }

        events
    }

    fn register_event(&mut self, event_object: &GameEventObject) -> bool {
        self.sent_events.insert(event_object.id.clone())
    }
}

pub struct MultiplayerEventConsumer {
    pub client: Rc<RefCell<MultiplayerConector>>,
}

impl EventConsumer for MultiplayerEventConsumer {
    fn handle_remote_event(&mut self, event: &GameEventObject) {
        let mut client = (*self.client).borrow_mut();
        let opponent_id = client.opponent_id.as_ref().unwrap().clone();
        if client.register_event(event) {
            client.client.send(event.clone(), opponent_id);
            debug!("Sent event: {:?}", event);
        }
    }

    fn handle_event(&mut self, event: &GameAction) {
        let sender = (*self.client)
            .borrow()
            .get_own_player_id()
            .expect("Own player ID unknown");
        self.handle_remote_event(&GameEventObject::new(event.clone(), &sender))
    }
}
