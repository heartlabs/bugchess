use std::{cell::RefCell, rc::Rc, collections::VecDeque};

use game_events::{event_broker::EventBroker, board_event_consumer::BoardEventConsumer, game_events::{GameEventObject, EventConsumer}};
use game_model::game::{Game, Team};
use game_render::{BoardRender, render_events::RenderEventConsumer};

use crate::{fakebox::FakeboxClient, multiplayer_connector::{MultiplayerConector, MultiplayerEventConsumer}};

pub struct TestGame {
    pub logs: Rc<RefCell<VecDeque<GameEventObject>>>, 
    pub game: Rc<RefCell<Game>>, 
    pub event_broker: EventBroker, 
    multiplayer_connector: Option<Rc<RefCell<MultiplayerConector>>>
}

impl TestGame {
    pub fn recieve_multiplayer_events(&mut self) {
        let recieved_events = (*self.multiplayer_connector.as_ref().unwrap())
            .borrow_mut()
            .try_recieve();
        recieved_events.iter()
            .for_each(|e| self.event_broker.handle_remote_event(&e));
    }
}

pub fn create_multiplayer_game() -> (TestGame, TestGame) {
    let (multiplayer_client1, multiplayer_client2) = FakeboxClient::new_client_pair();

    let sender_id1 = "1";
    let sender_id2 = "2";

    let mut test_game1 = create_singleplayer_game(sender_id1);
    make_multiplayer(sender_id1, multiplayer_client1, &mut test_game1);

    let mut test_game2 = create_singleplayer_game(sender_id2);
    make_multiplayer(sender_id2, multiplayer_client2, &mut test_game2);

    (test_game1, test_game2)
}


fn make_multiplayer(sender_id1: &str, multiplayer_client1: Rc<RefCell<FakeboxClient>>, test_game: &mut TestGame) {
    let mut multiplayer_connector = MultiplayerConector::new(sender_id1.to_string(), Box::new(multiplayer_client1));
    multiplayer_connector.matchmaking();
    let multiplayer_connector = Rc::new(RefCell::new(multiplayer_connector));
    let multiplayer_event_consumer = MultiplayerEventConsumer {client: multiplayer_connector.clone()};
    test_game.event_broker.subscribe(Box::new(multiplayer_event_consumer));
    test_game.multiplayer_connector = Some(multiplayer_connector);
}

pub fn create_singleplayer_game(sender_id1: &str) -> TestGame {
    let mut event_broker = EventBroker::new(sender_id1.to_string());
    let logs: Rc<RefCell<VecDeque<GameEventObject>>> = Rc::new(RefCell::new(VecDeque::new()));
    event_broker.subscribe(Box::new(EventLogger { events: logs.clone() }));
    let game = Rc::new(RefCell::new(create_game_object()));
    event_broker.subscribe(Box::new(BoardEventConsumer::new(sender_id1.to_string(), game.clone())));
    let board_render = BoardRender::new(&(*game.borrow()));
    event_broker.subscribe(Box::new(RenderEventConsumer::new(&Rc::new(RefCell::new(board_render)))));
    
    TestGame {
        event_broker, 
        logs, 
        game, 
        multiplayer_connector: None
    }
}

pub fn create_game_object() -> Game {
    let teams = vec![
        Team {
            name: "Red",
            id: 0,
            lost: false,
            unused_pieces: 2,
        },
        Team {
            name: "Yellow",
            id: 1,
            lost: false,
            unused_pieces: 2,
        },
    ];
    Game::new(teams, 8, 8)     
}

pub struct EventLogger {
    pub events: Rc<RefCell<VecDeque<GameEventObject>>>
}

impl EventConsumer for EventLogger {
    fn handle_event(&mut self, event: &GameEventObject) {
        (*self.events).borrow_mut().push_back(event.clone());
    }
}