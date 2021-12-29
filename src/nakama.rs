use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use nakama_rs::api_client::{ApiClient, Event};
use nakama_rs::matchmaker::{Matchmaker, QueryItemBuilder};
use nanoserde::DeBin;
use crate::{EventBroker, EventConsumer};
use crate::game_events::GameEventObject;
use crate::rand::rand;
use macroquad::prelude::*;

pub async fn nakama_client() -> Rc<RefCell<Box<NakamaClient>>> {
    let mut client = ApiClient::new("defaultkey", "34.65.62.178", 7350, "http");

    // Note that the minimum password length is 8 characters!
   // let uuid = Uuid::new_v4();
    let uuid = rand().to_string();

    client.register(&*(uuid.to_string() + "@user.com"), "password", &*uuid.to_string());

    wait_for_client(&mut client, "registration").await;

    if let Some(error) = client.error() {
        info!("Error: {:?}", error);
    }

    info!("Username: {:?}; {:?}; {:?}", client.username(), client.rpc_response(), client.authenticated());

    let mut matchmaker = Matchmaker::new();

    matchmaker
        .min(2)
        .max(2)
        .add_string_property("engine", "macroquad_matchmaking")
        .add_query_item(
            &QueryItemBuilder::new("engine")
                .required()
                .term("macroquad_matchmaking")
                .build(),
        );

    client.socket_add_matchmaker(&matchmaker);
    wait_for_client(&mut client, "matchmaking").await;

    while client.matchmaker_token.clone().is_none() {
        if let Some(error) = client.error().clone() {
            panic!("{}", error)
        }
        next_frame().await;
        client.tick();
    }

    if let Some(error) = client.error().clone() {
        panic!("{}", error)
    } else {
        let token = client.matchmaker_token.clone().unwrap();
        info!("Joining match with token {}", token);
        client.socket_join_match_by_token(&token);

    }

    wait_for_client(&mut client, "match joining").await;

    Rc::new(RefCell::new(Box::new(NakamaClient::new(client))))
}

async fn wait_for_client(client: &mut ApiClient, action: &str) {
    info!("Starting {}", action);
    while client.in_progress() {
        info!("{} in progress...", action);
        next_frame().await;
        client.tick();
    }
}

pub struct NakamaClient {
    sent_events: HashSet<String>,
    client: ApiClient
}

impl NakamaClient {
    pub fn new(client: ApiClient) -> Self {
        NakamaClient {
            sent_events: HashSet::new(),
            client
        }
    }

    pub fn try_recieve(&mut self, event_broker: &mut EventBroker) {
        self.client.tick();
        while let Some(event) = self.client.try_recv() {
            match event {
                Event::Presence { .. } => {}
                Event::MatchData {
                    data,
                    ..
                } => {
                    let event_object: GameEventObject = DeBin::deserialize_bin(&data).unwrap();
                    if self.register_event(&event_object) {
                        event_broker.handle_event(&event_object);
                    }
                }
            }
        }
    }

    fn register_event(&mut self, event_object: &GameEventObject) -> bool {
        self.sent_events.insert(event_object.id.clone())
    }
}

pub struct NakamaEventConsumer {
    pub(crate) nakama_client: Rc<RefCell<Box<NakamaClient>>>,
}


impl EventConsumer for NakamaEventConsumer {
    fn handle_event(&mut self, event: &GameEventObject) {
        let mut nakama_client = (*self.nakama_client).borrow_mut();
        if nakama_client.register_event(event) {
            nakama_client.client.socket_send(GameEventObject::OPCODE, event);
        }
    }
}