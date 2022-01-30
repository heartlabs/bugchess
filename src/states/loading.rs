use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, RawWaker, Waker};
use std::thread::Thread;
use futures::{Future, FutureExt, TryFutureExt};
use futures::future::{BoxFuture, LocalBoxFuture, OptionFuture};
use futures::task::LocalSpawnExt;
use crate::{Board, BoardEventConsumer, BoardRender, CompoundEventType, CoreGameState, EventBroker, GameEvent, GameState, nakama, NakamaClient, NakamaEventConsumer, ONLINE, Piece, PieceKind, Point2, Team};

use instant::Instant;
use macroquad::prelude::*;
use macroquad::rand::srand;
use nakama_rs::api_client::ApiClient;

pub struct LoadingState {
    core_game_state: Option<CoreGameState>,
    sub_state: LoadingSubState,
    client: Option<ApiClient>
}

#[derive(Debug, Copy, Clone)]
pub enum LoadingSubState {
    Register,
    Matchmaking,
    JoinMatch,
    Finished
}

impl LoadingState {
    pub fn new() -> Self {
        let start_time = Instant::now();

        let mut board = Rc::new(RefCell::new(Box::new(init_board())));
        let mut event_broker = EventBroker::new();
        event_broker.subscribe(Box::new(BoardEventConsumer {
            board: Rc::clone(&board),
        }));

        set_up_pieces(&mut board, &mut event_broker);

        let mut board_render = BoardRender::new((*board).borrow().as_ref());

        info!(
            "{}ns to set up pieces. {}",
            start_time.elapsed().as_nanos(),
            (*board).borrow().as_ref().teams[0].unused_pieces
        );

        srand((start_time.elapsed().as_nanos() % u64::MAX as u128) as u64);

        //let pool = futures::executor::LocalPool::new();
        //let s: Result<NakamaClient, E> = pool.spawner().spawn_local(nakama_client);

        LoadingState {
            core_game_state: Option::Some(CoreGameState::new(
                board,
                event_broker,
                board_render,
                Option::None)),
            sub_state: if ONLINE {LoadingSubState::Register} else {LoadingSubState::Finished},
            client: if ONLINE {Option::Some(nakama::start_registration())} else {Option::None},
        }
    }
}

impl GameState for LoadingState {
    fn update(&mut self) -> Option<Box<dyn GameState>> {
        match &self.sub_state {
            LoadingSubState::Register => {
                let client = self.client.as_mut().unwrap();
                if client.authenticated() == true {
                    nakama::add_matchmaker(client);
                    self.sub_state = LoadingSubState::Matchmaking;
                }
            }
            LoadingSubState::Matchmaking => {
                let client = self.client.as_mut().unwrap();
                if client.matchmaker_token.is_some() {
                    nakama::join_match(client);
                    self.sub_state = LoadingSubState::JoinMatch;
                }
            }
            LoadingSubState::JoinMatch => {
                if !self.client.as_ref().unwrap().in_progress() {
                    self.sub_state = LoadingSubState::Finished;
                }
            }
            LoadingSubState::Finished => {
                let mut core_game_state = self.core_game_state.take().unwrap();

                if ONLINE {
                    let nakama_client = NakamaClient::new(self.client.take().unwrap());
                    let nakama_events = Option::Some(Rc::new(RefCell::new(Box::new(nakama_client))));
                    core_game_state.event_broker.subscribe_committed(Box::new(NakamaEventConsumer {
                        nakama_client: Rc::clone(nakama_events.as_ref().unwrap()),
                    }));

                    core_game_state.nakama_events = nakama_events;
                }
                return Option::Some(Box::new(core_game_state))
            }
        }

        self.client.as_mut().unwrap().tick();

        Option::None
    }

    fn render(&self) {
        draw_text(
            &*format!("Loading: {:?}... ", self.sub_state),
            10.,
            400.,
            60.,
            GREEN,
        );
    }
}

fn set_up_pieces(board: &mut Rc<RefCell<Box<Board>>>, event_broker: &mut EventBroker) {
    let team_count = (**board).borrow().teams.len();

    let start_pieces = 4;

    for team_id in 0..team_count {
        let target_point = Point2::new((2 + team_id * 3) as u8, (2 + team_id * 3) as u8);
        let mut piece = Piece::new(team_id, PieceKind::Simple);
        piece.exhaustion.reset();
        event_broker.handle_new_event(&GameEvent::Place(target_point, piece));

        for _ in 0..start_pieces {
            event_broker.handle_new_event(&GameEvent::AddUnusedPiece(team_id));
        }
    }

    event_broker.commit(CompoundEventType::FinishTurn);
    event_broker.delete_history();
}

fn init_board() -> Board {
    let teams = vec![
        Team {
            name: "Red",
            id: 0,
            // color: Srgba::new(1., 1., 0.2, 1.),
            // color: Srgba::new(0.96,  0.49, 0.37, 1.),
            // color: Srgba::new(0.96, 0.37, 0.23, 1.),
            color: Color::new(0.76, 0.17, 0.10, 1.),
            lost: false,
            unused_pieces: 0,
        },
        Team {
            name: "Yellow",
            id: 1,
            // color: Srgba::new(0., 0., 0., 1.),
            // color: Srgba::new(0.93, 0.78, 0.31, 1.),
            color: Color::new(0.90, 0.68, 0.15, 1.),
            lost: false,
            unused_pieces: 0,
        },
    ];

    Board::new(teams)
}