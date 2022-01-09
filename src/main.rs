mod board;
mod constants;
mod game_events;
mod piece;
mod rendering;
mod states;
mod gui;
mod nakama;
mod conf;
mod ranges;

use crate::{
    game_events::{BoardEventConsumer, CompoundEventType, EventBroker, EventConsumer, GameEvent},
    board::*,
    constants::*,
    conf::*,
    piece::*,
    rendering::{BoardRender, CustomRenderContext},
    states::CoreGameState,
    nakama::NakamaEventConsumer,
    ranges::*,
};
use macroquad::prelude::*;
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::rc::Rc;
use macroquad::prelude::scene::RefMut;
use nakama_rs::api_client::ApiClient;
use futures::executor::block_on;
use std::collections::HashMap;
use instant::Instant;
use macroquad::rand::srand;
use nakama_rs::matchmaker::{Matchmaker, QueryItemBuilder};
use nanoserde::DeRonTok::Str;

//use wasm_bindgen::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Makrochess".to_owned(),
        window_width: 800,
        window_height: 800,
        ..Default::default()
    }
}




#[macroquad::main(window_conf)]
async fn main() {
    let start_time = Instant::now();
    const ONLINE: bool = true;

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

    srand((start_time.elapsed().as_nanos() % u64::MAX as u128) as u64);

    let mut nakama_events = Option::None;
    if ONLINE {
        nakama_events = Option::Some(nakama::nakama_client().await);
        event_broker.subscribe(Box::new(NakamaEventConsumer {
            nakama_client: Rc::clone(nakama_events.as_ref().unwrap())
        }));
    }
    info!("set up everything.");

    loop {
        if ONLINE {
            // TODO does that work?
            let x = nakama_events.as_mut().unwrap();
            (**x).borrow_mut().try_recieve(&mut event_broker);
        }
        board_render.render((*board).borrow().as_ref(), &render_context);

        if is_mouse_button_pressed(MouseButton::Left) {
            info!("mouse button pressed");
            let next_game_state = render_context.game_state.on_click(
                &cell_hovered(),
                board.as_ref().borrow_mut().borrow_mut(),
                &mut event_broker,
            );
            info!("{:?} -> {:?}", render_context.game_state.state, next_game_state.state);
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
            next_turn(&mut board, &mut event_broker);
        }

        if is_key_pressed(KeyCode::U) {
            event_broker.undo();
        }

        event_broker.flush();
        board_render = BoardRender::new((*board).borrow().as_ref());

        next_frame().await;
    }
}

fn next_turn(board: &mut Rc<RefCell<Box<Board>>>, event_broker: &mut EventBroker) {
    {
        let mut b = (**board).borrow_mut();
        let current_team_index = b.current_team_index;
        b.next_team();
        event_broker.handle_new_event(&GameEvent::AddUnusedPiece(current_team_index));
        event_broker.handle_new_event(&GameEvent::AddUnusedPiece(current_team_index));

        b.for_each_placed_piece_mut(|_point, mut piece| piece.exhaustion.reset());
    }
    event_broker.commit(Option::None);
    event_broker.delete_history();
}

fn set_up_pieces(board: &mut Rc<RefCell<Box<Board>>>, event_broker: &mut EventBroker) {
    let team_count = (**board).borrow().teams.len();

    let start_pieces = 2;

    for team_id in 0..team_count {
        let target_point = Point2::new((2 + team_id * 3) as u8, (2 + team_id * 3) as u8);
        let mut piece = Piece::new(team_id, PieceKind::Simple);
        piece.exhaustion.reset();
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
