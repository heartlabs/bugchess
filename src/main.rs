mod board;
mod constants;
mod game_events;
mod piece;
mod rendering;
mod states;
mod gui;
mod nakama;

use crate::game_events::{BoardEventConsumer, CompoundEventType, EventBroker, EventConsumer, GameEvent};
use crate::piece::{Piece, PieceKind};
use crate::rendering::{BoardRender, CustomRenderContext};
use crate::states::CoreGameState;
use crate::{
    board::*,
    constants::*,
    //    piece::*
};
use macroquad::prelude::*;
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::rc::Rc;
use macroquad::prelude::scene::RefMut;
use nakama_rs::api_client::ApiClient;

fn window_conf() -> Conf {
    Conf {
        window_title: "Makrochess".to_owned(),
        window_width: 800,
        window_height: 800,
        ..Default::default()
    }
}
/*
pub struct Nakama {
    pub api_client: ApiClient,
}

impl Nakama {
    pub fn new(key: &str, server: &str, port: u32, protocol: &str) -> Nakama {
        Nakama {
            api_client: ApiClient::new(key, server, port, protocol),
        }
    }
}

impl scene::Node for Nakama {
    fn ready(node: RefMut<Self>) {
        // Once created, nakama node should never be deleted.
        // The persist() call will make nakama node a singleton,
        // alive during all scene reloads.
        node.persist();
    }

    fn update(mut node: RefMut<Self>) {
        // api_client should be "ticked" once per frame
        node.api_client.tick();
    }
}*/

use futures::executor::block_on;
//use nakama_rs::api_client::ApiClient;
//use nakama_rs::default_client::DefaultClient;
use std::collections::HashMap;
use instant::Instant;
use macroquad::rand::srand;
use nakama_rs::matchmaker::{Matchmaker, QueryItemBuilder};
use crate::nakama::NakamaEventConsumer;

#[macroquad::main(window_conf)]
async fn main() {
    let start_time = Instant::now();

    let mut board = Rc::new(RefCell::new(Box::new(init_board())));
    let mut event_broker = EventBroker::new();
    event_broker.subscribe(Box::new(BoardEventConsumer {
        board: Rc::clone(&board),
    }));

    set_up_pieces(&mut board, &mut event_broker);

    info!(
        "set up pieces. {}",
        (*board).borrow().as_ref().teams[0].unused_pieces
    );

    let mut board_render = BoardRender::new((*board).borrow().as_ref());
    let mut render_context = CustomRenderContext::new();

    /*let nakama = scene::add_node(Nakama::new(
        "defaultkey", "127.0.0.1", 7350, "http"
    ));*/

    srand((start_time.elapsed().as_nanos() % u64::MAX as u128) as u64);
    let nakama_events = nakama::nakama_client().await;
    event_broker.subscribe(Box::new(NakamaEventConsumer {
        nakama_client: Rc::clone(&nakama_events)
    }));

    info!("set up everything.");
    loop {
        (*nakama_events).borrow_mut().try_recieve(&mut event_broker);
        board_render.render((*board).borrow().as_ref(), &render_context);

        if is_mouse_button_pressed(MouseButton::Left) {
            info!("mouse button pressed");
            let (x, y) = cell_hovered();
            let next_game_state = render_context.game_state.on_click(
                Point2::new(x, y),
                board.as_ref().borrow_mut().borrow_mut(),
                &mut event_broker,
            );
            render_context.game_state = next_game_state;
            render_context.reset_elapsed_time();

            if event_broker.flush() {
                info!("flushed");
                CoreGameState::merge_patterns((*board).borrow_mut().as_mut(), &mut event_broker);
                info!("committing");
                event_broker.commit(Some(CompoundEventType::Merge));
            }
            info!("finish mouse button action");
        }

        if is_key_pressed(KeyCode::N) {
            {
                let mut b = (*board).borrow_mut();
                let current_team_index = b.current_team_index;
                b.next_team();
                event_broker.handle_new_event(&GameEvent::AddUnusedPiece(current_team_index));
                event_broker.handle_new_event(&GameEvent::AddUnusedPiece(current_team_index));
            }
            event_broker.commit(Option::None);
            event_broker.delete_history();
        }

        if is_key_pressed(KeyCode::U) {
            event_broker.undo();
        }

        event_broker.flush();
        board_render = BoardRender::new((*board).borrow().as_ref());

        next_frame().await;
        info!("next frame");
    }
}

fn set_up_pieces(board: &mut Rc<RefCell<Box<Board>>>, event_broker: &mut EventBroker) {
    let team_count = (**board).borrow().teams.len();

    let start_pieces = 2;

    for team_id in 0..team_count {
        let target_point = Point2::new((2 + team_id * 3) as u8, (2 + team_id * 3) as u8);
        let piece = Piece::new(team_id, PieceKind::Simple);
        event_broker.handle_new_event(&GameEvent::Place(target_point, piece));

        for _ in 0..start_pieces {
            event_broker.handle_new_event(&GameEvent::AddUnusedPiece(team_id));
        }
    }

    event_broker.commit(None);
    event_broker.delete_history();
}

fn init_board() -> Board {
    let teams = vec![
        Team {
            name: "Unos",
            id: 0,
            // color: Srgba::new(1., 1., 0.2, 1.),
            // color: Srgba::new(0.96,  0.49, 0.37, 1.),
            // color: Srgba::new(0.96, 0.37, 0.23, 1.),
            color: Color::new(0.76, 0.17, 0.10, 1.),
            lost: false,
            unused_pieces: 0,
        },
        Team {
            name: "Duos",
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
