use game_core::multiplayer_connector::{MultiplayerClient, MultiplayerConector};
use macroquad::prelude::*;
use matchbox_socket::{ChannelConfig, RtcIceServerConfig, WebRtcSocket, WebRtcSocketConfig};
use nanoserde::{DeJson, SerJson};
use urlencoding::encode;
use game_core::game_events::GameEventObject;

fn connect(room_id: &str) -> MatchboxClient {
    let (socket, loop_fut) = WebRtcSocket::new_with_config(WebRtcSocketConfig {
        room_url: format!("wss://heartlabs.tech:3537/{}?next=2", encode(room_id)),
        ice_server: RtcIceServerConfig {
            urls: vec![
                "stun:heartlabs.tech:3478".to_string(),
                "turn:heartlabs.tech:3478".to_string(),
            ],
            username: Some("testuser".to_string()),
            credential: Some("fyUTdD7dQjeSauYv".to_string()), // does it make sense to hide this better?
        },
        channels: vec![ChannelConfig::reliable()],
    });

    info!("my id is {:?}", socket.id());

    #[cfg(target_family = "wasm")]
    wasm_bindgen_futures::spawn_local(loop_fut);

    #[cfg(not(target_family = "wasm"))]
    std::thread::spawn(move || {
        let executor = async_executor::Executor::new();
        let task = executor.spawn(loop_fut);
        futures::executor::block_on(executor.run(task));
        error!("message future quit");
    });

    MatchboxClient { socket }
}
pub struct MatchboxClient {
    socket: WebRtcSocket,
}

impl MultiplayerClient for MatchboxClient {
    fn is_ready(&self) -> bool {
        debug!("{:?}", self.socket.connected_peers());
        self.socket.connected_peers().len() == 1
    }

    fn accept_new_connections(&mut self) -> Vec<String> {
        self.socket.accept_new_connections()
    }

    fn recieved_events(&mut self) -> Vec<GameEventObject> {
        let new_connections = self.accept_new_connections(); // TODO: make sure this is necessary

        if !new_connections.is_empty() {
            debug!("New player connected: {:?}", new_connections);
        }

        self.socket
            .receive()
            .into_iter()
            .map(|(_, g)| {
                let json = std::str::from_utf8(&(*g)).unwrap();
                DeJson::deserialize_json(json).unwrap()
            }) // TODO: Proper error
            .collect()
    }

    fn send(&mut self, game_object: &GameEventObject, opponent_id: &str) {
        debug!("Sending {} to {}", game_object, opponent_id);

        let json = game_object.serialize_json();
        self.socket.send(
            json.into_bytes().into_boxed_slice(),
            opponent_id.to_string(),
        );
    }

    fn own_player_id(&self) -> Option<String> {
        Some(self.socket.id().clone())
    }
}

impl MatchboxClient {
    pub fn new_connector(room_id: &str) -> MultiplayerConector {
        let client = connect(room_id);
        MultiplayerConector::new(Box::new(client))
    }
}
