use crate::{
    matchbox, Board, BoardEventConsumer, BoardRender, CompoundEventType, CoreGameState,
    EventBroker, GameEvent, GameState, ONLINE,
};
use futures::future::{BoxFuture, LocalBoxFuture, OptionFuture};
use futures::task::LocalSpawnExt;
use futures::{Future, FutureExt, TryFutureExt};
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, RawWaker, Waker};
use std::thread::Thread;

use crate::game_events::RenderEventConsumer;
use crate::matchbox::{MatchboxClient, MatchboxEventConsumer};
use crate::states::core_game_state::CoreGameSubstate;
use instant::Instant;
use macroquad::prelude::*;
use macroquad::rand::srand;
use macroquad_canvas::Canvas2D;
use matchbox_socket::WebRtcSocket;
use crate::game_logic::{
    board::*,
    game::*,
    piece::*,
};

pub struct LoadingState {
    core_game_state: Option<CoreGameState>,
    sub_state: LoadingSubState,
    client: Option<MatchboxClient>,
}

#[derive(Debug, Copy, Clone)]
pub enum LoadingSubState {
    Register,
    Matchmaking,
    JoinMatch,
    Finished,
}

impl LoadingState {
    pub fn new() -> Self {
        let start_time = Instant::now();

        let mut game = Rc::new(RefCell::new(Box::new(init_game())));
        let mut event_broker = EventBroker::new();
        event_broker.subscribe(Box::new(BoardEventConsumer {
            game: Rc::clone(&game),
        }));

        let num_teams = (*game).borrow().as_ref().teams.len();
        set_up_pieces(num_teams, &mut event_broker);

        let mut board_render = Rc::new(RefCell::new(Box::new(BoardRender::new(
            (*game).borrow().as_ref(),
        ))));
        //TODO event_broker.subscribe(Box::new(RenderEventConsumer { board_render: board_render.clone() }));

        info!(
            "{}ns to set up pieces. {}",
            start_time.elapsed().as_nanos(),
            (*game).borrow().as_ref().teams[0].unused_pieces
        );

        srand((start_time.elapsed().as_nanos() % u64::MAX as u128) as u64);

        //let pool = futures::executor::LocalPool::new();
        //let s: Result<NakamaClient, E> = pool.spawner().spawn_local(nakama_client);

        LoadingState {
            core_game_state: Option::Some(CoreGameState::new(
                game,
                event_broker,
                board_render,
                Option::None,
            )),
            sub_state: if ONLINE {
                LoadingSubState::Register
            } else {
                LoadingSubState::Finished
            },
            client: Option::None,
        }
    }
}

impl GameState for LoadingState {
    fn update(&mut self, _canvas: &Canvas2D) -> Option<Box<dyn GameState>> {
        match &self.sub_state {
            LoadingSubState::Register => {
                let client = matchbox::connect();
                self.client = Some(client);
                self.sub_state = LoadingSubState::Matchmaking;
            }
            LoadingSubState::Matchmaking => {
                let client = self.client.as_mut().unwrap();
                client.matchmaking();

                if client.is_ready() {
                    self.sub_state = LoadingSubState::JoinMatch;
                }
            }
            LoadingSubState::JoinMatch => {
                self.sub_state = LoadingSubState::Finished;
            }
            LoadingSubState::Finished => {
                let mut core_game_state = self.core_game_state.take().unwrap();

                if ONLINE {
                    let matchbox_client = self.client.take().unwrap();

                    if matchbox_client.get_own_player_index().unwrap() != 0 {
                        core_game_state.set_sub_state(CoreGameSubstate::Wait);
                    }

                    let matchbox_events =
                        Option::Some(Rc::new(RefCell::new(Box::new(matchbox_client))));
                    core_game_state.event_broker.subscribe_committed(Box::new(
                        MatchboxEventConsumer {
                            client: Rc::clone(matchbox_events.as_ref().unwrap()),
                        },
                    ));

                    core_game_state.matchbox_events = matchbox_events;
                }
                return Option::Some(Box::new(core_game_state));
            }
        }

        Option::None
    }

    fn render(&self, _canvas: &Canvas2D) {
        draw_text(
            &*format!("Loading: {:?}... ", self.sub_state),
            10.,
            400.,
            60.,
            GREEN,
        );
    }
}

fn set_up_pieces(team_count: usize, event_broker: &mut EventBroker) {
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

fn init_game() -> Game {
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

    Game::new(teams)
}
