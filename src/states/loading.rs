use crate::{
    Board, BoardEventConsumer, BoardRender, CompoundEventType, CoreGameState, EventBroker,
    GameEvent, GameState, matchbox, ONLINE,
};
use futures::{
    future::{BoxFuture, LocalBoxFuture, OptionFuture},
    Future,
    FutureExt, task::LocalSpawnExt, TryFutureExt,
};
use std::{
    borrow::BorrowMut,
    cell::RefCell,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, RawWaker, Waker},
    thread::Thread,
};
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};

use crate::{
    game_logic::{board::*, game::*, piece::*},
    matchbox::{MatchboxClient, MatchboxEventConsumer},
    states::core_game_state::CoreGameSubstate,
};
use instant::Instant;
use macroquad::{prelude::*, rand::srand};
use macroquad::ui::Drag::No;
use macroquad_canvas::Canvas2D;
use matchbox_socket::WebRtcSocket;
use uuid::Uuid;
use crate::game_events::EventComposer;
use crate::rendering::render_events::RenderEventConsumer;

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
    WaitForOpponent,
}

impl Display for LoadingSubState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display_name = match self {
            LoadingSubState::Register => "Register",
            LoadingSubState::Matchmaking => "Matchmaking",
            LoadingSubState::JoinMatch => "Joining Match",
            LoadingSubState::WaitForOpponent => "Wait for Opponent"
        };

        write!(f, "{}", display_name)
    }
}

impl LoadingState {
    pub fn new() -> Self {
        let start_time = Instant::now();

        let mut game = Rc::new(RefCell::new(Box::new(init_game())));
        let own_sender_id = Uuid::new_v4().to_string();
        let mut event_broker = EventBroker::new(own_sender_id.clone());
        let mut event_composer = EventComposer::new(Rc::clone(&game));
        event_broker.subscribe_committed(Box::new(BoardEventConsumer {
            own_sender_id,
            game: Rc::clone(&game),
        }));

        let mut board_render = Rc::new(RefCell::new(Box::new(BoardRender::new(
            (*game).borrow().as_ref(),
        ))));
        event_broker.subscribe_committed(Box::new(RenderEventConsumer { board_render: board_render.clone() }));

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
                event_composer,
                board_render,
                Option::None,
            )),
            sub_state: if ONLINE {
                LoadingSubState::Register
            } else {
                LoadingSubState::WaitForOpponent
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
                let mut core_game_state = self.core_game_state.as_mut().unwrap();

                let matchbox_client = self.client.take().unwrap();

                if matchbox_client.get_own_player_index().unwrap() != 0 {
                    core_game_state.set_sub_state(CoreGameSubstate::Wait);
                } else {
                    let num_teams = 2;
                    set_up_pieces(num_teams, &mut core_game_state.event_composer);
                }

                let matchbox_events =
                    Option::Some(Rc::new(RefCell::new(Box::new(matchbox_client))));
                core_game_state.event_broker.subscribe_committed(Box::new(
                    MatchboxEventConsumer {
                        client: Rc::clone(matchbox_events.as_ref().unwrap()),
                    },
                ));

                core_game_state.matchbox_events = matchbox_events;

                self.sub_state = LoadingSubState::WaitForOpponent;

            }
            LoadingSubState::WaitForOpponent => {

                if ONLINE {
                    let mut core_game_state = self.core_game_state.as_mut().unwrap();
                    let wait_for_opponent = {
                        let client = core_game_state.matchbox_events.as_ref().unwrap().as_ref().borrow();
                        client.get_own_player_index().unwrap() != 0 && client.recieved_events.is_empty()
                    };
                    if wait_for_opponent {
                        let events = {
                            let mut client = core_game_state.matchbox_events.as_ref().unwrap().as_ref().borrow_mut();
                            client.try_recieve()
                        };
                        events.iter()
                            .for_each(|e| core_game_state.event_broker.handle_remote_event(e));
                        return None;
                    }

                } else {
                    let mut core_game_state = self.core_game_state.as_mut().unwrap();
                    let num_teams = 2;
                    set_up_pieces(num_teams, &mut core_game_state.event_composer);
                }
                return Option::Some(Box::new(self.core_game_state.take().unwrap()));
            }
        }

        Option::None
    }

    fn render(&self, _canvas: &Canvas2D) {
        draw_text(
            &*format!("Loading: {}... ", self.sub_state),
            10.,
            400.,
            60.,
            GREEN,
        );
    }
}

fn set_up_pieces(team_count: usize, event_composer: &mut EventComposer) {
    let start_pieces = 8;

    event_composer.start_transaction(CompoundEventType::FinishTurn);

    for team_id in 0..team_count {
        let target_point = Point2::new((2 + team_id * 3) as u8, (2 + team_id * 3) as u8);
        let mut piece = Piece::new(team_id, PieceKind::Simple);
        piece.exhaustion.reset();
        event_composer.push_event(GameEvent::Place(target_point, piece));

        for _ in 0..start_pieces {
            event_composer.push_event(GameEvent::AddUnusedPiece(team_id));
        }
    }

    event_composer.commit();
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
