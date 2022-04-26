use crate::game_events::EventConsumer;
use crate::{game_events::GameEventObject, rand::rand, EventBroker};
use macroquad::prelude::*;
use matchbox_socket::{RtcIceServerConfig, WebRtcSocket, WebRtcSocketConfig};
use nanoserde::{DeBin, SerBin};
use std::future::Future;
use std::pin::Pin;
use std::{cell::RefCell, collections::HashSet, rc::Rc};

use async_executor::LocalExecutor;
use futures::FutureExt;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn getStunUrl() -> String;
}

pub fn connect() -> MatchboxClient {
    info!("Stun URL: {}", getStunUrl());
    let (mut socket, loop_fut) = WebRtcSocket::new_with_config(WebRtcSocketConfig {
        room_url: "wss://heartlabs.tech:3537/example_room?next=2"
            .parse()
            .unwrap(),
        ice_server: RtcIceServerConfig { urls: vec![getStunUrl()] },
    });
    //let (mut socket, loop_fut) = WebRtcSocket::new("wss://heartlabs.tech:3537/example_room?next=2");

    info!("my id is {:?}", socket.id());

    //let loop_fut = loop_fut.fuse();
    //futures::pin_mut!(loop_fut);

    MatchboxClient::new(socket, loop_fut)

    /*    let timeout = Delay::new(Duration::from_millis(100));
    futures::pin_mut!(timeout);*/
}

/*
let mut server = String::from("34.65.62.178");
    let mut protocol = String::from("http");
    let mut port = 7350;

    let config: Box<dyn MyObject> = get_configuration();
    if let Some(s) = config
        .get_field("nakama")
        .and_then(|n| n.get_field("server"))
    {
        s.to_string(&mut server);
    }
    if let Some(s) = config
        .get_field("nakama")
        .and_then(|n| n.get_field("protocol"))
    {
        s.to_string(&mut protocol);
    }
    if let Some(s) = config.get_field("nakama").and_then(|n| n.get_field("port")) {
        let mut port_str: String = String::new();
        s.to_string(&mut port_str);
        port = port_str.parse().unwrap();
    }

    info!("Got {}, {}, {}", protocol, server, port);
    */

pub struct MatchboxClient {
    sent_events: HashSet<String>,
    client: WebRtcSocket,
    own_player_id: String,
    pub(crate) opponent_id: Option<String>,
    //executor: LocalExecutor<'static>,
}

impl MatchboxClient {
    pub fn new(client: WebRtcSocket, message_loop: Pin<Box<dyn Future<Output = ()>>>) -> Self {
        let own_player_id = client.id().to_string();
        let client = MatchboxClient {
            sent_events: HashSet::new(),
            client,
            own_player_id,
            opponent_id: None,
            //executor:  LocalExecutor::new()
        };

        //client.executor.spawn(message_loop).detach();
        #[cfg(target_family = "wasm")]
        wasm_bindgen_futures::spawn_local(message_loop);

        client
    }

    pub(crate) fn is_ready(&self) -> bool {
        self.client.connected_peers().len() == 1
    }

    pub fn matchmaking(&mut self) {
        for peer in self.client.accept_new_connections() {
            info!("Found a peer {:?}", peer);
            self.opponent_id = Some(peer);
        }
    }

    pub fn get_own_player_index(&self) -> Option<usize> {
        if let Some(opponent_id) = self.opponent_id.as_ref() {
            if *opponent_id < self.own_player_id {
                return Some(1);
            }

            return Some(0);
        }

        return None;
    }

    pub fn try_recieve(&mut self, event_broker: &mut EventBroker) {
        for (_peer, packet) in self.client.receive() {
            let event_object: GameEventObject = DeBin::deserialize_bin(&packet).unwrap();
            if self.register_event(&event_object) {
                event_broker.handle_remote_event(&event_object);
            }
        }
    }

    fn register_event(&mut self, event_object: &GameEventObject) -> bool {
        self.sent_events.insert(event_object.id.clone())
    }
}

pub struct MatchboxEventConsumer {
    pub(crate) client: Rc<RefCell<Box<MatchboxClient>>>,
}

impl EventConsumer for MatchboxEventConsumer {
    fn handle_event(&mut self, event: &GameEventObject) {
        let mut client = (*self.client).borrow_mut();
        let opponent_id = client.opponent_id.as_ref().unwrap().clone();
        if client.register_event(event) {
            client
                .client
                .send(event.serialize_bin().into_boxed_slice(), opponent_id);
        }
    }
}
