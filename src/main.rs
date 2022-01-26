mod board;
mod conf;
mod constants;
mod game_events;
mod nakama;
mod piece;
mod ranges;
mod rendering;
mod states;

use crate::{
    board::*,
    conf::*,
    constants::*,
    game_events::{BoardEventConsumer, CompoundEventType, EventBroker, EventConsumer, GameEvent},
    nakama::NakamaEventConsumer,
    piece::*,
    ranges::*,
    rendering::{BoardRender, CustomRenderContext},
    states::{CoreGameState, State},
};

use instant::Instant;
use macroquad::{prelude::*, rand::srand};

use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    rc::Rc,
};

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
    #[cfg(target_family = "wasm")]
    const ONLINE: bool = true;
    #[cfg(not(target_family = "wasm"))]
    const ONLINE: bool = false;

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
    let mut own_player_id = Option::None;
    if ONLINE {
        let nakama_client = nakama::nakama_client().await;
        nakama_events = Option::Some(Rc::new(RefCell::new(Box::new(nakama_client))));
        event_broker.subscribe_committed(Box::new(NakamaEventConsumer {
            nakama_client: Rc::clone(nakama_events.as_ref().unwrap()),
        }));
    }
    info!("set up everything.");

    loop {
        if ONLINE {
            let x = nakama_events.as_mut().unwrap();
            (**x).borrow_mut().try_recieve(&mut event_broker);

            if own_player_id.is_none() {
                own_player_id = (**x).borrow_mut().get_own_player_index();
                info!("own player id: {:?}", own_player_id);
            }
        }
        board_render.render((*board).borrow().as_ref(), &render_context);

        if can_control_player((*board).borrow().as_ref(), &mut own_player_id, ONLINE) {
            for (i, text) in description(&render_context, (*board).borrow().as_ref())
                .iter()
                .enumerate()
            {
                draw_text(
                    text.as_str(),
                    10.,
                    670. + (i * 50) as f32,
                    50.,
                    (*board).borrow().as_ref().current_team().color,
                );
            }

            handle_player_input(&mut board, &mut event_broker, &mut render_context);
        } else {
            draw_text(
                "Please wait for opponent to finish",
                10.,
                670.,
                50.,
                (*board).borrow().as_ref().current_team().color,
            );
        }

        check_if_somebody_won((*board).borrow().as_ref(), &mut render_context);

        board_render = BoardRender::new((*board).borrow().as_ref());

        next_frame().await;
    }
}

fn check_if_somebody_won(board: &Board, render_context: &mut CustomRenderContext) {
    if board.placed_pieces(0).is_empty() || board.num_unused_pieces_of(1) >= 20 {
        info!("Team 1 won");
        render_context.game_state = CoreGameState::won(1);
    }
    if board.placed_pieces(1).is_empty() || board.num_unused_pieces_of(0) >= 20 {
        info!("Team 0 won");
        render_context.game_state = CoreGameState::won(0);
    }
}

fn description(render_context: &CustomRenderContext, board: &Board) -> Vec<String> {
    let mut description = vec![];

    let CoreGameState { selected, state: _ } = &render_context.game_state;

    match render_context.game_state.state {
        State::Place => {
            let all_pieces_exhausted = board
                .placed_pieces(board.current_team_index)
                .iter()
                .all(|&piece| !piece.can_use_special() && !piece.can_move());

            if board.unused_piece_available() {
                description.push("Click on a square to place a piece".parse().unwrap());
            }

            if !all_pieces_exhausted {
                description.push("Click on a piece to use it".parse().unwrap());
            }

            if description.is_empty() {
                description.push("Press 'N' to end your turn".parse().unwrap());
            }
        }
        State::Move => {
            description.push("Click target square to move it".parse().unwrap());

            if board
                .get_piece_at(selected.as_ref().unwrap())
                .unwrap()
                .can_use_special()
            {
                description.push("Click it again for special power".parse().unwrap());
            }
        }
        State::Activate => {
            description.push("Click the target piece".parse().unwrap());
        }
        State::Won(team) => {
            description.push(
                format!("The {} team won", board.teams[team].name)
                    .parse()
                    .unwrap(),
            );
        }
    };

    description
}

fn can_control_player(board: &Board, own_player_id: &mut Option<usize>, is_online: bool) -> bool {
    if !is_online {
        return true;
    }

    if let Some(pid) = own_player_id {
        *pid == board.current_team_index
    } else {
        false
    }
}

fn handle_player_input(
    mut board: &mut Rc<RefCell<Box<Board>>>,
    mut event_broker: &mut EventBroker,
    render_context: &mut CustomRenderContext,
) {
    if is_mouse_button_pressed(MouseButton::Left) {
        info!("mouse button pressed");
        let next_game_state = render_context.game_state.on_click(
            &cell_hovered(),
            board.as_ref().borrow_mut().borrow_mut(),
            &mut event_broker,
        );
        info!(
            "{:?} -> {:?}",
            render_context.game_state.state, next_game_state.state
        );
        render_context.game_state = next_game_state;
        render_context.reset_elapsed_time();

        event_broker.flush();
        CoreGameState::merge_patterns((**board).borrow_mut().as_mut(), &mut event_broker);
        event_broker.commit(CompoundEventType::Merge);
        info!("finish mouse button action");
    }

    if is_key_pressed(KeyCode::N) {
        next_turn(&mut board, &mut event_broker);
    }

    if is_key_pressed(KeyCode::U) {
        event_broker.undo();
    }
}

fn next_turn(board: &mut Rc<RefCell<Box<Board>>>, event_broker: &mut EventBroker) {
    {
        let mut b = (**board).borrow_mut();
        let current_team_index = b.current_team_index;
        event_broker.handle_new_event(&GameEvent::NextTurn);
        event_broker.handle_new_event(&GameEvent::AddUnusedPiece(current_team_index));
        event_broker.handle_new_event(&GameEvent::AddUnusedPiece(current_team_index));

        b.for_each_placed_piece_mut(|_point, piece| piece.exhaustion.reset());
    }
    event_broker.commit(CompoundEventType::FinishTurn);
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
