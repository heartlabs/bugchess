use super::fakebox::FakeboxClient;
use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use game_core::{
    board_event_consumer::BoardEventConsumer, command_handler::CommandHandler,
    core_game::CoreGameSubstate, game_controller::GameCommand,
    multiplayer_connector::MultiplayerConector,
};
use game_events::{
    actions::compound_events::GameAction,
    event_broker::{EventBroker, EventConsumer},
};
use game_model::{
    game::{Game, Team},
    piece::PieceKind,
};
use game_render::{render_events::RenderEventConsumer, BoardRender};

pub struct TestGame {
    pub logs: Rc<RefCell<VecDeque<GameAction>>>,
    pub game: Rc<RefCell<Game>>,
    pub command_handler: CommandHandler,
    pub game_state: CoreGameSubstate,
    multiplayer_connector: Option<Rc<RefCell<MultiplayerConector>>>,
}

impl TestGame {
    pub fn add_unused_pieces(&mut self, t1: u8, t2: u8) {
        let teams = &mut (*self.game).borrow_mut().teams;
        teams[0].unused_pieces += t1;
        teams[1].unused_pieces += t2;
    }

    pub fn recieve_multiplayer_events(&mut self) {
        let recieved_events = (*self.multiplayer_connector.as_ref().unwrap())
            .borrow_mut()
            .try_recieve();

        println!("RECEIVED {} EVENTS", recieved_events.len());

        recieved_events.iter().for_each(|e| {
            let game = (*self.game.borrow()).clone();
            self.command_handler.handle_remote_command(game, e)
        });
    }

    pub fn click_at_pos(&mut self, pos: (u8, u8)) {
        let game_clone = (*self.game.borrow()).clone();
        self.game_state =
            self.game_state
                .on_click(&pos.into(), game_clone, &mut self.command_handler);
    }

    pub fn next_turn(&mut self) {
        let game = (*self.game).borrow().clone();
        self.command_handler
            .handle_new_command(game, &GameCommand::NextTurn);
    }

    pub fn assert_has_game_state(&self, game_state: CoreGameSubstate) {
        assert_eq!(self.game_state, game_state);
    }

    pub fn assert_num_pieces(&self, num_pieces_team_1: usize, num_pieces_team_2: usize) {
        let game = &(*self.game.borrow());

        assert_eq!(game.board.placed_pieces(0).len(), num_pieces_team_1);
        assert_eq!(game.board.placed_pieces(1).len(), num_pieces_team_2);
    }

    pub fn assert_piece_at(&self, piece_pos: (u8, u8), piece_kind: PieceKind) {
        let game = &(*self.game.borrow());
        let placed_piece = game
            .board
            .get_piece_at(&piece_pos.into())
            .expect("Placed piece not found on board");
        assert_eq!(placed_piece.piece_kind, piece_kind);
    }
}

pub fn create_multiplayer_game() -> (TestGame, TestGame) {
    let (multiplayer_client1, multiplayer_client2) = FakeboxClient::new_client_pair();

    let mut test_game1 = create_singleplayer_game();
    make_multiplayer(multiplayer_client1, &mut test_game1);

    let mut test_game2 = create_singleplayer_game();
    make_multiplayer(multiplayer_client2, &mut test_game2);

    test_game1.add_unused_pieces(3, 3);
    test_game2.add_unused_pieces(3, 3);
    (test_game1, test_game2)
}

fn make_multiplayer(multiplayer_client1: Rc<RefCell<FakeboxClient>>, test_game: &mut TestGame) {
    let mut multiplayer_connector = MultiplayerConector::new(Box::new(multiplayer_client1));
    multiplayer_connector.matchmaking();
    let multiplayer_connector = Rc::new(RefCell::new(multiplayer_connector));

    test_game.command_handler.multiplayer_connector = Some(multiplayer_connector.clone());
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
    let board_render = BoardRender::new(&game.borrow());
    event_broker.subscribe(Box::new(RenderEventConsumer::new(&Rc::new(RefCell::new(
        board_render,
    )))));

    TestGame {
        command_handler: CommandHandler::new(event_broker),
        logs,
        game,
        game_state: CoreGameSubstate::Place,
        multiplayer_connector: None,
    }
}

pub fn create_game_object() -> Game {
    let teams = vec![
        Team {
            id: 0,
            lost: false,
            unused_pieces: 0,
        },
        Team {
            id: 1,
            lost: false,
            unused_pieces: 0,
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
