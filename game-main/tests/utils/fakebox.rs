use game_core::game_events::GameEventObject;
use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use game_core::multiplayer_connector::MultiplayerClient;

pub struct FakeboxClient {
    id: String,
    incoming_messages: VecDeque<GameEventObject>,
    opponent_client: Option<Rc<RefCell<FakeboxClient>>>,
}

impl FakeboxClient {
    pub fn new(id: &str) -> Self {
        FakeboxClient {
            id: id.to_string(),
            incoming_messages: VecDeque::new(),
            opponent_client: None,
        }
    }
    pub fn new_client_pair() -> (Rc<RefCell<Self>>, Rc<RefCell<Self>>) {
        let client1 = FakeboxClient::new("1");
        let client1 = Rc::new(RefCell::new(client1));

        let mut client2 = FakeboxClient::new("2");
        client2.connect(client1.clone());
        let client2 = Rc::new(RefCell::new(client2));

        (*client1).borrow_mut().connect(client2.clone());

        (client1, client2)
    }

    pub fn disconnect(&mut self) {
        if let Some(opponent) = &self.opponent_client {
            opponent.borrow_mut().opponent_client = None;
        }
        self.opponent_client = None;
    }

    pub fn connect(&mut self, other: Rc<RefCell<FakeboxClient>>) {
        self.opponent_client = Some(other);
    }
}

impl MultiplayerClient for FakeboxClient {
    fn is_ready(&self) -> bool {
        true
    }

    fn accept_new_connections(&mut self) -> Vec<String> {
        if let Some(client) = &self.opponent_client {
            vec![client.borrow().id.clone()]
        } else {
            vec![]
        }
    }

    fn recieved_events(&mut self) -> Vec<GameEventObject> {
        self.incoming_messages.drain(..).collect()
    }

    fn send(&mut self, game_object: &GameEventObject, _opponent_id: &str) {
        let opponent_client = self
            .opponent_client
            .as_ref()
            .expect("Can't send: No opponent's client");
        (*opponent_client)
            .borrow_mut()
            .incoming_messages
            .push_back(game_object.clone());
    }

    fn own_player_id(&self) -> Option<String> {
        Some(self.id.clone())
    }
}
