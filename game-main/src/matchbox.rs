use crate::multiplayer_connector::{MultiplayerClient, MultiplayerConector};
use game_events::game_events::GameEventObject;
use macroquad::prelude::*;
use matchbox_socket::{RtcIceServerConfig, WebRtcSocket, WebRtcSocketConfig};
use nanoserde::{DeBin, SerBin};
use std::{borrow::Borrow, future::Future, pin::Pin};
use urlencoding::encode;

#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
extern "C" {
    fn getStunUrl() -> String;
}

#[cfg(not(target_family = "wasm"))]
fn getStunUrl() -> String {
    "".to_string()
}

fn connect(room_id: &str) -> MatchboxClient {
    info!("Stun URL currently ignored: {}", getStunUrl());
    let (socket, loop_fut) = WebRtcSocket::new_with_config(WebRtcSocketConfig {
        room_url: format!("wss://heartlabs.tech:3537/{}?next=2", encode(room_id)),
        ice_server: RtcIceServerConfig {
            //urls: vec![getStunUrl()],
            urls: vec!["stun:heartlabs.tech:3478".to_string(), "turn:heartlabs.tech:3478".to_string()],
            username: Some("testuser".to_string()),
            credential: Some("fyUTdD7dQjeSauYv".to_string()), // does it make sense to hide this better?
        },
    });

    info!("my id is {:?}", socket.id());

    MatchboxClient::new(socket, loop_fut)
}
pub struct MatchboxClient {
    client: WebRtcSocket,
    //executor: LocalExecutor<'static>,
}

impl MultiplayerClient for MatchboxClient {
    fn is_ready(&self) -> bool {
        debug!("{:?}", self.client.connected_peers());
        self.client.connected_peers().len() == 1
    }

    fn accept_new_connections(&mut self) -> Vec<String> {
        self.client.accept_new_connections()
    }

    fn recieved_events(&mut self) -> Vec<GameEventObject> {
        self.client
            .receive()
            .into_iter()
            .map(|(_, g)| DeBin::deserialize_bin(&g).unwrap()) // TODO: Proper error
            .collect()
    }

    fn send(&mut self, game_object: GameEventObject, opponent_id: String) {
        self.client
            .send(game_object.serialize_bin().into_boxed_slice(), opponent_id)
    }

    fn own_player_id(&self) -> Option<String> {
        Some(self.client.id().clone())
    }
}

impl MatchboxClient {
    pub fn new(client: WebRtcSocket, message_loop: Pin<Box<dyn Future<Output = ()>>>) -> Self {
        let client = MatchboxClient { client };

        //client.executor.spawn(message_loop).detach();
        #[cfg(target_family = "wasm")]
        wasm_bindgen_futures::spawn_local(message_loop);

        client
    }

    pub fn new_connector(room_id: &str) -> MultiplayerConector {
        let client = connect(room_id);
        MultiplayerConector::new(Box::new(client))
    }
}
