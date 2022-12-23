use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use game_core::{
    core_game::CoreGameSubstate, event_broker::EventBroker, fakebox::FakeboxClient,
    game_controller::GameController, multiplayer_connector::MultiplayerConector,
};
use game_events::{
    actions::compound_events::GameAction, board_event_consumer::BoardEventConsumer,
    game_events::EventConsumer,
};
use game_model::game::{Game, Team};
use game_render::{render_events::RenderEventConsumer, BoardRender};

pub struct TestGame {
    pub logs: Rc<RefCell<VecDeque<GameAction>>>,
    pub game: Rc<RefCell<Game>>,
    pub event_broker: EventBroker,
    multiplayer_connector: Option<Rc<RefCell<MultiplayerConector>>>,
}

impl TestGame {
    pub fn recieve_multiplayer_events(&mut self) {
        let recieved_events = (*self.multiplayer_connector.as_ref().unwrap())
            .borrow_mut()
            .try_recieve();

        println!("RECEIVED {} EVENTS", recieved_events.len());

        recieved_events
            .iter()
            .for_each(|e| self.event_broker.handle_remote_event(&e));
    }

    pub fn click_at_pos(
        &mut self,
        pos: (u8, u8),
        game_state: CoreGameSubstate,
    ) -> CoreGameSubstate {
        let game_clone = (*self.game.borrow()).clone();
        game_state.on_click(&pos.into(), game_clone, &mut self.event_broker)
    }

    pub fn next_turn(&mut self) -> CoreGameSubstate {
        let event_option = GameController::next_turn(&(*self.game).borrow());
        self.event_broker.handle_new_event(&event_option);

        CoreGameSubstate::Wait
    }
}

pub fn create_multiplayer_game() -> (TestGame, TestGame) {
    let (multiplayer_client1, multiplayer_client2) = FakeboxClient::new_client_pair();

    let mut test_game1 = create_singleplayer_game();
    make_multiplayer(multiplayer_client1, &mut test_game1);

    let mut test_game2 = create_singleplayer_game();
    make_multiplayer(multiplayer_client2, &mut test_game2);

    (test_game1, test_game2)
}

fn make_multiplayer(multiplayer_client1: Rc<RefCell<FakeboxClient>>, test_game: &mut TestGame) {
    let mut multiplayer_connector = MultiplayerConector::new(Box::new(multiplayer_client1));
    multiplayer_connector.matchmaking();
    let multiplayer_connector = Rc::new(RefCell::new(multiplayer_connector));

    test_game.event_broker.multiplayer_connector = Some(multiplayer_connector.clone());
    test_game.multiplayer_connector = Some(multiplayer_connector);
}

pub fn create_singleplayer_game() -> TestGame {
    let mut event_broker = EventBroker::new();
    let logs: Rc<RefCell<VecDeque<GameAction>>> = Rc::new(RefCell::new(VecDeque::new()));
    event_broker.subscribe(Box::new(EventLogger {
        events: logs.clone(),
    }));
    let game = Rc::new(RefCell::new(create_game_object()));
    event_broker.subscribe(Box::new(BoardEventConsumer::new(game.clone())));
    let board_render = BoardRender::new(&(*game.borrow()));
    event_broker.subscribe(Box::new(RenderEventConsumer::new(&Rc::new(RefCell::new(
        board_render,
    )))));

    TestGame {
        event_broker,
        logs,
        game,
        multiplayer_connector: None,
    }
}

pub fn create_game_object() -> Game {
    let teams = vec![
        Team {
            name: "Red",
            id: 0,
            lost: false,
            unused_pieces: 3,
        },
        Team {
            name: "Yellow",
            id: 1,
            lost: false,
            unused_pieces: 3,
        },
    ];
    Game::new(teams, 8, 8)
}

pub struct EventLogger {
    pub events: Rc<RefCell<VecDeque<GameAction>>>,
}

impl EventConsumer for EventLogger {
    fn handle_event(&mut self, event: &GameAction) {
        (*self.events).borrow_mut().push_back(event.clone());
    }
}
