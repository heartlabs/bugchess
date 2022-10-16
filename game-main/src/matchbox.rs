use game_events::game_events::{EventConsumer, GameEventObject};
use macroquad::prelude::*;
use matchbox_socket::{RtcIceServerConfig, WebRtcSocket, WebRtcSocketConfig};
use nanoserde::{DeBin, SerBin};
use std::{cell::RefCell, collections::HashSet, future::Future, pin::Pin, rc::Rc, borrow::Borrow};
use crate::{multiplayer_connector::{MultiplayerClient, MultiplayerConector},};
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
    info!("Stun URL: {}", getStunUrl());
    let (socket, loop_fut) = WebRtcSocket::new_with_config(WebRtcSocketConfig {
        room_url: format!("wss://heartlabs.tech:3537/{}?next=2", encode(room_id)),
        ice_server: RtcIceServerConfig {
            urls: vec![getStunUrl()],
        },
    });
    //let (mut socket, loop_fut) = WebRtcSocket::new("wss://heartlabs.tech:3537/example_room?next=2");

    info!("my id is {:?}", socket.id());

    //let loop_fut = loop_fut.fuse();
    //futures::pin_mut!(loop_fut);

    MatchboxClient::new(socket, loop_fut)

    /*    let timeout = Delay::new(Duration::from_millis(100));
    futures::pin_mut!(timeout);*/
}
pub struct MatchboxClient {
    client: WebRtcSocket,
    //executor: LocalExecutor<'static>,
}

impl MultiplayerClient for MatchboxClient {
    fn is_ready(&self) -> bool {
        self.client.connected_peers().len() == 1
    }

    fn accept_new_connections(&mut self) -> Vec<String> {
        self.client.accept_new_connections()
    }

    fn recieved_events(&mut self) -> Vec<GameEventObject> {
        self.client.receive()
            .into_iter()
            .map(|(_, g)| DeBin::deserialize_bin(g.borrow()).unwrap()) // TODO: Proper error
            .collect()
    }

    fn send(&mut self, game_object: GameEventObject, opponent_id: String ) {
        self.client.send(game_object.serialize_bin().into_boxed_slice(), opponent_id)
    }
}

impl MatchboxClient {
    pub fn new(client: WebRtcSocket, message_loop: Pin<Box<dyn Future<Output = ()>>>) -> Self {
        let client = MatchboxClient {
            client,
        };

        //client.executor.spawn(message_loop).detach();
        #[cfg(target_family = "wasm")]
        wasm_bindgen_futures::spawn_local(message_loop);

        client
    }

    pub fn new_connector(room_id: &str) -> MultiplayerConector {
        let client = connect(room_id);
        MultiplayerConector::new(client.client.id().clone(), Box::new(client))
    }
      
}
