use crate::multiplayer_connector::{MultiplayerClient, MultiplayerConector};
use egui_macroquad::egui::mutex::Mutex;
use futures::SinkExt;
use game_events::game_events::GameEventObject;
use macroquad::prelude::*;
use matchbox_socket::{RtcIceServerConfig, WebRtcSocket, WebRtcSocketConfig};
use nanoserde::{DeBin, SerBin};
use tokio::net::TcpStream;
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream, tungstenite::Message};
use std::{borrow::Borrow, future::Future, pin::Pin, sync::Arc, collections::VecDeque};
use url::Url;
use futures_util::{future, pin_mut, StreamExt};

#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
extern "C" {
    fn getCustomServerUrl() -> String;
}

#[cfg(not(target_family = "wasm"))]
fn getCustomServerUrl() -> String {
    "".to_string()
}

async fn connect(room_id: &str) -> CustomClient {
    info!("Server URL: {}", getCustomServerUrl());
    
    let (socket, response) = tokio_tungstenite::connect_async(
        Url::parse(getCustomServerUrl().as_str()).unwrap()
    ).await.expect("Can't connect");

    let (write, read) = socket.split();

    let recieved_messages = Arc::new(Mutex::new(VecDeque::new()));

    let rmc = recieved_messages.clone();

    tokio::spawn(async move {
        read.for_each(|result| async {
            let message = result.unwrap();
            let queue = (*rmc).lock().push_back(message);
        });

    });

    CustomClient { 
        recieved_messages,
        write
    }
}
pub struct CustomClient {
    //socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    recieved_messages: Arc<Mutex<VecDeque<Message>>>,
    write: futures::stream::SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>
}

#[derive(DeBin, SerBin, Clone, Debug)]
pub enum CustomMessage {
    Connect(String),
    Event(GameEventObject)
}

impl CustomClient {
    fn recieve(&mut self) -> Option<CustomMessage> {
        for message in (*self.recieved_messages).lock().iter() {
            match message {
                Message::Text(string) => panic!("String messages not supported: {}", string),
                Message::Ping(_) | Message::Pong(_)| Message::Close(_) | Message::Frame(_) => {},
                Message::Binary(message) => return DeBin::deserialize_bin(message.as_slice()).expect("Could not deserialize ws message"),
            }
        } 

        None
    }

    fn send_message(&mut self, message: CustomMessage) {
        self.write.start_send_unpin(Message::Binary(message.serialize_bin())).unwrap();
    }
}

impl MultiplayerClient for CustomClient {
    fn is_ready(&self) -> bool {
        true
    }

    fn accept_new_connections(&mut self) -> Vec<String> {
        /*let t = thread::spawn(|| self.recieve());
        if t.is_finished() {
            if let CustomMessage::Connect(id) = t.join().unwrap() {
                return vec![id];
            } else {
                panic!("Expected connect message");
            }
        }*/
       
        return vec![];
    }

    fn recieved_events(&mut self) -> Vec<GameEventObject> {
        /*let t = thread::spawn(|| self.recieve());
        if t.is_finished() {
            if let CustomMessage::Event(id) = t.join().unwrap() {
                return vec![id];
            } else {
                panic!("Expected connect message");
            }
        }
       */
        return vec![];
    }

    fn send(&mut self, game_object: GameEventObject, _opponent_id: String) {
        self.send_message(CustomMessage::Event(game_object));
    }

    fn own_player_id(&self) -> Option<String> {
        todo!()
    }
}

