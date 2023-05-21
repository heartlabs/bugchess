use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    fs::File,
    io::Write,
    rc::Rc,
    sync::{Arc, Mutex},
};

use game_core::{core_game::CoreGameSubstate, multiplayer_connector::MultiplayerConector};
use game_model::game::Game;

use game_render::{constants::cell_hovered, BoardRender, CustomRenderContext};

use crate::states::GameState;
use game_core::{command_handler::CommandHandler, game_controller::GameCommand};
use game_events::event_broker::EventBroker;

use macroquad::prelude::*;
use macroquad_canvas::Canvas2D;
use nanoserde::SerJson;

pub struct CoreGameState {
    pub game: Rc<RefCell<Game>>,
    pub(crate) command_handler: CommandHandler,
    board_render: Rc<RefCell<BoardRender>>,
    pub matchbox_events: Option<Rc<RefCell<MultiplayerConector>>>,
    render_context: CustomRenderContext,
    own_player_team_id: Option<usize>,
    pub is_multi_player: bool,
    pub team_names: Vec<String>,
}

impl CoreGameState {
    pub(crate) fn new(
        game: Rc<RefCell<Game>>,
        event_broker: EventBroker,
        board_render: Rc<RefCell<BoardRender>>,
        matchbox_events: Option<Rc<RefCell<MultiplayerConector>>>,
        is_multi_player: bool,
        team_names: Vec<String>,
    ) -> Self {
        let commands = Arc::new(Mutex::new(vec![]));
        let command_handler = CommandHandler::new(event_broker, commands.clone());

        let once = std::sync::Once::new();

        std::panic::set_hook(Box::new(move |panic_info| {
            let message = panic_info
                .payload()
                .downcast_ref::<&str>()
                .unwrap_or(&"Panicked without error string");

            let _location = panic_info
                .location()
                .map(|l| l.to_string())
                .unwrap_or("Unknown Location".to_string());

            error!("{} after commands: {:?}", panic_info, (*commands).lock());

            once.call_once(|| {
                let commands = (*commands).lock().unwrap().borrow().to_vec();

                #[cfg(not(target_family = "wasm"))]
                if let Err(e) = export_to_file(message, &commands) {
                    println!("{:?}", e);
                }

                #[cfg(target_family = "wasm")]
                {
                    let error_report_url = web_sys::window()
                        .as_ref()
                        .and_then(web_sys::Window::document)
                        .and_then(|document| document.url().ok())
                        .and_then(|url| url::Url::parse(&url).ok())
                        .and_then(|url| Some(format!("{}://{}", url.scheme(), url.host_str()?)))
                        .unwrap();
                    wasm_bindgen_futures::spawn_local(post_error_report(
                        error_report_url,
                        format!("{}", panic_info),
                        commands,
                    ))
                }
            });
        }));

        CoreGameState {
            game,
            command_handler,
            board_render,
            matchbox_events,
            render_context: CustomRenderContext::new(),
            own_player_team_id: None,
            is_multi_player,
            team_names,
        }
    }

    pub fn game_clone(&self) -> Game {
        (*self.game).borrow().clone()
    }

    pub fn set_sub_state(&mut self, sub_state: CoreGameSubstate) {
        self.render_context.game_state = sub_state;
    }

    fn update_internal(&mut self, canvas: &Canvas2D) -> Option<Box<dyn GameState>> {
        if self.is_multi_player {
            let recieved_events = (**self.matchbox_events.as_mut().unwrap())
                .borrow_mut()
                .try_recieve();

            recieved_events.iter().for_each(|e| {
                self.command_handler
                    .handle_remote_command(self.game_clone(), e)
            });

            if self.own_player_team_id.is_none() {
                self.own_player_team_id = (**self.matchbox_events.as_ref().unwrap())
                    .borrow()
                    .get_own_player_index();
                info!("own player id: {:?}", self.own_player_team_id);
            }
        }

        match self.render_context.game_state {
            CoreGameSubstate::Wait => {
                if can_control_player(
                    &(*self.game).borrow(),
                    &self.own_player_team_id,
                    self.is_multi_player,
                ) {
                    self.render_context.game_state = CoreGameSubstate::Place;
                }
            }
            CoreGameSubstate::Won(_) => {}
            _ => {
                handle_player_input(
                    &mut self.game,
                    &mut self.command_handler,
                    &mut self.render_context,
                    canvas,
                );
            }
        }

        check_if_somebody_won(&(*self.game).borrow(), &mut self.render_context);

        (*self.board_render).borrow_mut().update();

        Option::None
    }

    fn render_internal(&self, canvas: &Canvas2D) {
        let board_render = (*self.board_render).borrow();
        let game = (*self.game).borrow();
        board_render.render(&(*self.game).borrow().board, &self.render_context, canvas);

        for (i, text) in description(&self.render_context, &game, &self.team_names)
            .iter()
            .enumerate()
        {
            draw_text(
                text.as_str(),
                10.,
                670. + (i * 50) as f32,
                50.,
                *board_render.get_team_color(game.current_team_index),
            );
        }
    }
}
/*
fn catch_unwind_silent<F: FnOnce() -> R + std::panic::UnwindSafe, R>(f: F) -> Result<R, GameError> {
    let mut error_holder: Option<GameError> = None;

    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|panic_info| {
        let message = panic_info
            .payload()
            .downcast_ref::<&str>()
            .unwrap_or(&"Panicked without error string");
        let message = format!(
            "Panicked at {} with message {}",
            panic_info.location().unwrap(),
            message
        );

        let _ = error_holder.insert(GameError::new(message));
    }));
    let result = std::panic::catch_unwind(f);
    std::panic::set_hook(prev_hook);

    if let Ok(r) = result {
        Ok(r)
    } else {
        Err(error_holder.unwrap_or(GameError::new("Unknown Error".to_string())))
    }
}
*/
impl GameState for CoreGameState {
    fn update(&mut self, canvas: &Canvas2D) -> Option<Box<dyn GameState>> {
        self.update_internal(canvas)
    }

    fn render(&self, canvas: &Canvas2D) {
        self.render_internal(canvas)
    }

    fn uses_egui(&self) -> bool {
        false
    }
}

fn check_if_somebody_won(game: &Game, render_context: &mut CustomRenderContext) {
    let board = &game.board;
    let team_1_won = board.placed_pieces(0).is_empty() || game.num_unused_pieces_of(1) >= 20;
    let team_0_won = board.placed_pieces(1).is_empty() || game.num_unused_pieces_of(0) >= 20;

    if team_1_won && !team_0_won {
        info!("Team 1 won");
        render_context.game_state = CoreGameSubstate::Won(1);
    }
    if team_0_won && !team_1_won {
        info!("Team 0 won");
        render_context.game_state = CoreGameSubstate::Won(0);
    }
}

fn description(
    render_context: &CustomRenderContext,
    game: &Game,
    team_names: &[String],
) -> Vec<String> {
    let mut description = vec![];
    let board = &game.board;

    match render_context.game_state {
        CoreGameSubstate::Place => {
            let all_pieces_exhausted = board
                .placed_pieces(game.current_team_index)
                .iter()
                .all(|&piece| !piece.can_use_special() && !piece.can_move());

            if game.unused_piece_available() {
                description.push("Click on a square to place a piece".parse().unwrap());
            }

            if !all_pieces_exhausted {
                description.push("Click on a piece to use it".parse().unwrap());
            }

            if description.is_empty() {
                description.push("Click \"End Turn\" or press ENTER".parse().unwrap());
            }
        }
        CoreGameSubstate::Move(selected) => {
            description.push("Click target square to move it".parse().unwrap());

            if board.get_piece_at(&selected).unwrap().can_use_special() {
                description.push("Click it again for special power".parse().unwrap());
            }
        }
        CoreGameSubstate::Activate(_) => {
            description.push("Click the target piece".parse().unwrap());
        }
        CoreGameSubstate::Won(team) => {
            description.push(
                format!("The {} team won", team_names[team])
                    .parse()
                    .unwrap(),
            );
        }
        CoreGameSubstate::Wait => {
            description.push("Please wait for opponent to finish".parse().unwrap());
        }
    };

    description
}

fn can_control_player(game: &Game, own_player_id: &Option<usize>, is_online: bool) -> bool {
    if !is_online {
        return true;
    }

    if let Some(pid) = own_player_id {
        *pid == game.current_team_index
    } else {
        false
    }
}

fn handle_player_input(
    game: &mut Rc<RefCell<Game>>,
    command_handler: &mut CommandHandler,
    render_context: &mut CustomRenderContext,
    canvas: &Canvas2D,
) {
    if is_key_pressed(KeyCode::U) || render_context.button_undo.clicked(canvas) {
        let game_clone = (**game).borrow().clone();
        command_handler.handle_new_command(game_clone, &GameCommand::Undo);
    } else if is_key_pressed(KeyCode::D) {
        if let Err(e) = export_to_file("exported_game", command_handler.get_past_commands()) {
            error!("Could not export game to file: {:?}", e);
        }
    } else if is_key_pressed(KeyCode::Enter)
        || is_key_pressed(KeyCode::KpEnter)
        || render_context.button_next.clicked(canvas)
    {
        let game_clone = (**game).borrow().clone();
        command_handler.handle_new_command(game_clone, &GameCommand::NextTurn);

        render_context.game_state = CoreGameSubstate::Wait;
        // BoardEventConsumer::flush_unsafe(game.as_ref().borrow_mut().borrow_mut(), &event_option);
    } else if is_mouse_button_pressed(MouseButton::Left) {
        let game_clone = (**game).borrow().clone();
        let next_game_state =
            render_context
                .game_state
                .on_click(&cell_hovered(canvas), game_clone, command_handler);

        info!("{:?} -> {:?}", render_context.game_state, next_game_state);
        render_context.game_state = next_game_state;
    }
}

const EXPORTED_GAMES_DIR: &str = "game-main/tests/exported_games";

fn export_to_file(message: &str, content: &Vec<GameCommand>) -> Result<(), std::io::Error> {
    let num_games = std::fs::read_dir(EXPORTED_GAMES_DIR)?.count();
    let filename = format!(
        "{}/{:04}_{}.json",
        EXPORTED_GAMES_DIR,
        num_games + 1,
        message.replace(['/', '\\'], "_").as_str()
    );

    println!("Exporting to {}", filename);
    let mut file = File::create(filename)?;
    file.write(format!("// {}\n", message).as_ref())?;
    file.write(content.serialize_json().into_bytes().as_slice())?;

    Ok(())
}

async fn post_error_report(url: String, message: String, content: Vec<GameCommand>) {
    let client = reqwest::Client::new();
    if let Err(e) = client
        .post(url + ":3030/error_report")
        .body(format!("// {}\n\n{}", message, content.serialize_json()))
        .send()
        .await
    {
        println!("{:?}", e);
    }
}

#[cfg(test)]
mod tests {}
