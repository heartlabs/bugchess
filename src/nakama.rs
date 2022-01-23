use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use nakama_rs::api_client::{ApiClient, Event};
use nakama_rs::matchmaker::{Matchmaker, QueryItemBuilder};
use nanoserde::DeBin;
use crate::{EventBroker, EventConsumer, get_configuration, MyObject};
use crate::game_events::GameEventObject;
use crate::rand::rand;
use macroquad::prelude::*;
use nakama_rs::rt_api::Presence;

pub async fn nakama_client() -> NakamaClient {
    //let mut server = String::from("127.0.0.1");
    let mut server = String::from("34.65.62.178");
    let mut protocol = String::from("http");
    let mut port = 7350;

    let config: Box<dyn MyObject> = get_configuration();
    if let Some(s) = config.get_field("nakama").and_then(|n| n.get_field("server")) {
        s.to_string(&mut server);
    }
    if let Some(s) = config.get_field("nakama").and_then(|n| n.get_field("protocol")) {
        s.to_string(&mut protocol);
    }
    if let Some(s) = config.get_field("nakama").and_then(|n| n.get_field("port")) {
        let mut port_str: String = String::new();
        s.to_string(&mut port_str);
        port = port_str.parse().unwrap();
    }


    info!("Got {}, {}, {}", protocol, server, port);

    let mut client = ApiClient::new("defaultkey", server.as_str(), port, protocol.as_str());

    // Note that the minimum password length is 8 characters!
   // let uuid = Uuid::new_v4();
    let uuid = rand().to_string();

    client.register(&*(uuid.to_string() + "@user.com"), "password", &*uuid.to_string());

    wait_for_client(&mut client, "registration").await;

    if let Some(error) = client.error() {
        info!("Error: {:?}", error);
    }

    while client.authenticated() == false {
        info!("Waiting for authentication");
        next_frame().await;
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

    NakamaClient::new(client)
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
    client: ApiClient,
    players: Vec<Presence>
}

impl NakamaClient {
    pub fn new(client: ApiClient) -> Self {
        NakamaClient {
            sent_events: HashSet::new(),
            client,
            players: vec![]
        }
    }

    pub fn get_own_player_index(&self) -> Option<usize> {
        if self.players.len() != 2 {
            return None
        }

        let sort_ascending = self.client.matchmaker_token.as_ref().unwrap().as_bytes()[0] % 2 == 0;

        let mut sorted = self.players.clone();
        sorted.sort_by_key(|p| p.user_id.clone());

        if sort_ascending {
            sorted.reverse();
        }

        self.client.username().and_then(|u| {
            for (i,p) in sorted.iter().enumerate() {
                if *u == p.username {
                    return Option::Some(i)
                }
            }
            Option::None
        })

    }

    pub fn try_recieve(&mut self, event_broker: &mut EventBroker) {
        self.client.tick();
        while let Some(event) = self.client.try_recv() {
            match event {
                Event::Presence { joins, leaves } => {
                    for presence in joins {
                        info!("Joined: {} ({}, {})", presence.username, presence.session_id, presence.user_id);

                        self.players.push(presence);
                    }

                    for presence in leaves {
                        error!("Left: {} ({}, {})", presence.username, presence.session_id, presence.user_id)
                    }

                }
                Event::MatchData {
                    data,
                    ..
                } => {
                    let event_object: GameEventObject = DeBin::deserialize_bin(&data).unwrap();
                    if self.register_event(&event_object) {
                        event_broker.handle_remote_event(&event_object);
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