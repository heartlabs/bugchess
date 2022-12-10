use game_events::{game_events::{GameEventObject}};
use macroquad::{prelude::{info, error}, rand::rand};
use nakama_rs::{api_client::{ApiClient, Event}, matchmaker::{Matchmaker, QueryItemBuilder}, rt_api::Presence};
use nanoserde::DeBin;

use url::Url;
#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

use crate::multiplayer_connector::{MultiplayerClient, MultiplayerConector};

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
extern "C" {
    fn getNakamaUrl() -> String;
}

#[cfg(not(target_family = "wasm"))]
fn getNakamaUrl() -> String {
    "".to_string()
}

fn join_match(client: &mut ApiClient) {
    if let Some(error) = client.error().clone() {
        panic!("{}", error)
    } else {
        let token = client.matchmaker_token.clone().unwrap();
        info!("Joining match with token {}", token);
        client.socket_join_match_by_token(&token);
    }
}

fn add_matchmaker(client: &mut ApiClient) {
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
}

pub fn start_registration() -> ApiClient {
//let mut server = String::from("127.0.0.1");
    
    let mut server = String::from("34.65.62.178");
    let mut protocol = String::from("http");
    let mut port = 7350;

    if let Ok(s) = Url::parse(getNakamaUrl().as_str())
    {
        server = s.host_str()
            .map(|s| s.to_string())
            .unwrap_or(server);
    
        protocol = s.scheme().to_string();

        port = s.port().map(|p|p.into()).unwrap_or(port);

    }

    info!("Got {}, {}, {}", protocol, server, port);

    let mut client = ApiClient::new("defaultkey", server.as_str(), port, protocol.as_str());

    // Note that the minimum password length is 8 characters!
    // let uuid = Uuid::new_v4();
    let uuid = rand().to_string();

    client.register(
        &*(uuid.to_string() + "@user.com"),
        "password",
        &*uuid.to_string(),
    );
    client
}

pub struct NakamaClient {
    client: ApiClient,
    players: Vec<Presence>,
}

impl NakamaClient {
    pub fn new(client: ApiClient) -> Self {
        NakamaClient {
            client,
            players: vec![],
        }
    }
}

// doesn't actually work and difficult to debug
impl MultiplayerClient for NakamaClient {
    fn is_ready(&self) -> bool {
        !self.client.in_progress() && self.client.authenticated()
    }

    // doesn't actually work and difficult to debug
    fn accept_new_connections(&mut self) -> Vec<String> {
        info!("{:?}, {:?}, {:?}, {:?}", self.client.username(), self.client.in_progress(), self.client.session_id, self.client.error());
        self.client.tick();
        info!("after tick: {:?}, {:?}, {:?}, {:?}", self.client.username(), self.client.in_progress(), self.client.session_id, self.client.error());
        if self.is_ready() && self.client.matchmaker_token.is_none()  && self.players.is_empty() {
            add_matchmaker(&mut self.client);
            join_match(&mut self.client);
        }
        self.players.iter()
            .map(|p| p.user_id.clone())
            .collect()
    }

    fn recieved_events(&mut self) -> Vec<GameEventObject> {
        self.client.tick();
        let mut events = vec![];
        while let Some(event) = self.client.try_recv() {
            match event {
                Event::Presence { joins, leaves } => {
                    for presence in joins {
                        info!(
                            "Joined: {} ({}, {})",
                            presence.username, presence.session_id, presence.user_id
                        );

                        self.players.push(presence);
                    }

                    for presence in leaves {
                        error!(
                            "Left: {} ({}, {})",
                            presence.username, presence.session_id, presence.user_id
                        )
                    }
                }
                Event::MatchData { data, .. } => {
                    let event_object: GameEventObject = DeBin::deserialize_bin(&data).unwrap();
                    events.push(event_object);
                }
            }
        }

        events
    }

    fn send(&mut self, game_object: GameEventObject, opponent_id: String) {
        self
            .client
            .socket_send(GameEventObject::OPCODE, &game_object);
    }

    fn own_player_id(&self) -> Option<String> {
        self.client.session_id.clone()
    }
}
