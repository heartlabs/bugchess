use game_core::{
    game_events::GameEventObject,
    multiplayer_connector::{MultiplayerClient, MultiplayerConector},
};
use macroquad::prelude::*;
use matchbox_socket::{PeerId, PeerState, RtcIceServerConfig, WebRtcSocket};
use nanoserde::{DeJson, SerJson};
use urlencoding::encode;

fn connect(room_id: &str) -> MatchboxClient {
    let (mut socket, loop_fut) =
        WebRtcSocket::builder(format!("wss://heartlabs.eu:3537/{}", encode(room_id)))
            .ice_server(RtcIceServerConfig {
                urls: vec![
                    "stun:heartlabs.eu:3478".to_string(),
                    "turn:heartlabs.eu:3478".to_string(),
                ],
                username: Some("testuser".to_string()),
                credential: Some("fyUTdD7dQjeSauYv".to_string()), // does it make sense to hide this better?
            })
            .add_reliable_channel()
            .build();

    info!("my id is {:?}", socket.id());

    let loop_fut_simple = async {
        info!("matchbox loop started");
        loop_fut.await.unwrap();
        error!("matchbox loop exited");
    };

    #[cfg(target_family = "wasm")]
    wasm_bindgen_futures::spawn_local(loop_fut_simple);

    #[cfg(not(target_family = "wasm"))]
    std::thread::spawn(move || {
        let executor = async_executor::Executor::new();
        let task = executor.spawn(loop_fut_simple);
        futures::executor::block_on(executor.run(task));
        error!("message future quit");
    });

    MatchboxClient {
        socket,
        is_ready: false,
        own_id: None,
    }
}
pub struct MatchboxClient {
    socket: WebRtcSocket,
    is_ready: bool,
    own_id: Option<String>,
}

impl MultiplayerClient for MatchboxClient {
    fn is_ready(&self) -> bool {
        self.is_ready
    }

    fn accept_new_connections(&mut self) -> Vec<String> {
        // Refresh own_id while we have &mut self (socket.id() requires &mut self in 0.14+)
        if self.own_id.is_none() {
            self.own_id = self.socket.id().map(|id| id.0.to_string());
        }
        self.socket
            .update_peers()
            .into_iter()
            .filter(|(_, state)| *state == PeerState::Connected)
            .map(|(i, _)| {
                self.is_ready = true;
                i.0.to_string()
            })
            .collect()
    }

    fn recieved_events(&mut self) -> Vec<GameEventObject> {
        self.socket
            .channel_mut(0)
            .receive()
            .into_iter()
            .map(|(_, g)| {
                let json = std::str::from_utf8(&g).unwrap();
                DeJson::deserialize_json(json).unwrap()
            }) // TODO: Proper error
            .collect()
    }

    fn send(&mut self, game_object: &GameEventObject, opponent_id: &str) {
        debug!("Sending {} to {}", game_object, opponent_id);

        let json = game_object.serialize_json();
        self.socket.channel_mut(0).send(
            json.into_bytes().into_boxed_slice(),
            PeerId(opponent_id.try_into().unwrap()),
        );
    }

    fn own_player_id(&self) -> Option<String> {
        self.own_id.clone()
    }
}

impl MatchboxClient {
    pub fn new_connector(room_id: &str) -> MultiplayerConector {
        let client = connect(room_id);
        MultiplayerConector::new(Box::new(client))
    }
}
