mod board;
mod constants;
mod piece;
mod rendering;
mod states;
mod game_events;

use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::rc::Rc;
use macroquad::prelude::*;
use crate::{
    board::*,
    constants::*,
//    piece::*
};
use crate::game_events::{BoardEventConsumer, CompoundEventType, EventBroker, EventConsumer, GameEvent};
use crate::piece::{Piece, PieceKind};
use crate::rendering::{BoardRender, CustomRenderContext};
use crate::states::CoreGameState;

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
    let mut board = Rc::new(RefCell::new(Box::new(init_board())));
    let mut event_broker = EventBroker::new();
    event_broker.subscribe(Box::new(BoardEventConsumer {board: Rc::clone(&board)}));

    set_up_pieces(&mut board, &mut event_broker);

    println!("set up pieces. {}", (*board).borrow().as_ref().teams[0].unused_pieces);

    let mut board_render = BoardRender::new((*board).borrow().as_ref());
    let mut render_context = CustomRenderContext::new();

    println!("set up everything.");
    loop {
        board_render.render((*board).borrow().as_ref(), &render_context);

        if is_mouse_button_pressed(MouseButton::Left) {
            let (x, y) = cell_hovered();
            let next_game_state = render_context.game_state.on_click(Point2::new(x,y), board.as_ref().borrow_mut().borrow_mut(), &mut event_broker);
            render_context.game_state = next_game_state;
            render_context.reset_elapsed_time();
            if event_broker.flush() {
                CoreGameState::merge_patterns((*board).borrow_mut().as_mut(), &mut event_broker);
                event_broker.commit(Some(CompoundEventType::Merge));
            }
        }

        if is_key_pressed(KeyCode::N) {
            {
                let mut b = (*board).borrow_mut();
                let current_team_index = b.current_team_index;
                b.next_team();
                event_broker.handle_event(&GameEvent::AddUnusedPiece(current_team_index));
                event_broker.handle_event(&GameEvent::AddUnusedPiece(current_team_index));
            }
            event_broker.commit(Option::None);
            event_broker.delete_history();
        }

        if is_key_pressed(KeyCode::U) {
            event_broker.undo();
        }

        event_broker.flush();
        board_render = BoardRender::new((*board).borrow().as_ref());

        next_frame().await
    }
}

fn set_up_pieces(board: &mut Rc<RefCell<Box<Board>>>, event_broker: &mut EventBroker) {
    let team_count = (**board).borrow().teams.len();

    let start_pieces = 2;

    for team_id in 0..team_count {
        let target_point = Point2::new((2 + team_id * 3) as u8, (2 + team_id * 3) as u8);
        let piece = Piece::new(team_id, PieceKind::Simple);
        event_broker.handle_event(&GameEvent::Place(target_point, piece));

        for _ in 0..start_pieces {
            event_broker.handle_event(&GameEvent::AddUnusedPiece(team_id));
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
            unused_pieces: 0
        },
        Team {
            name: "Duos",
            id: 1,
            // color: Srgba::new(0., 0., 0., 1.),
            // color: Srgba::new(0.93, 0.78, 0.31, 1.),
            color: Color::new(0.90, 0.68, 0.15, 1.),
            lost: false,
            unused_pieces: 0
        },
    ];

    Board::new(teams)
}