use game_core::{
    game_events::GameEventObject,
    multiplayer_connector::{MultiplayerClient, MultiplayerConector},
};
use macroquad::prelude::*;
use matchbox_socket::{ChannelConfig, PeerId, RtcIceServerConfig, WebRtcSocket};
use nanoserde::{DeJson, SerJson};
use urlencoding::encode;

fn connect(room_id: &str) -> MatchboxClient {
    let (socket, loop_fut) =
        WebRtcSocket::builder(format!("ws://localhost:3537/{}?next=2", encode(room_id)))
            /*.ice_server(RtcIceServerConfig {
                urls: vec![
                    //"stun:heartlabs.tech:3478".to_string(),
                    //"turn:heartlabs.tech:3478".to_string(),
                ],
                username: Some("testuser".to_string()),
                credential: Some("fyUTdD7dQjeSauYv".to_string()), // does it make sense to hide this better?
            })*/
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
    }
}
pub struct MatchboxClient {
    socket: WebRtcSocket,
    is_ready: bool,
}

impl MultiplayerClient for MatchboxClient {
    fn is_ready(&self) -> bool {
        self.is_ready
    }

    fn accept_new_connections(&mut self) -> Vec<String> {
        self.socket
            .update_peers()
            .iter()
            .map(|(i, _)| {
                self.is_ready = true;
                i.0.to_string()
            })
            .collect()
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
                let json = std::str::from_utf8(&g).unwrap();
                DeJson::deserialize_json(json).unwrap()
            }) // TODO: Proper error
            .collect()
    }

    fn send(&mut self, game_object: &GameEventObject, opponent_id: &str) {
        debug!("Sending {} to {}", game_object, opponent_id);

        let json = game_object.serialize_json();
        self.socket.send(
            json.into_bytes().into_boxed_slice(),
            PeerId(opponent_id.try_into().unwrap()),
        );
    }

    fn own_player_id(&self) -> Option<String> {
        self.socket.id().map(|id| id.0.to_string())
    }
}

impl MatchboxClient {
    pub fn new_connector(room_id: &str) -> MultiplayerConector {
        let client = connect(room_id);
        MultiplayerConector::new(Box::new(client))
    }
}
