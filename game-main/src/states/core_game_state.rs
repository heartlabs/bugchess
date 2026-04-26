use std::{
    borrow::Borrow,
    cell::RefCell,
    fs::File,
    io::Write,
    rc::Rc,
    sync::{Arc, Mutex},
};

use game_core::{core_game::CoreGameSubstate, multiplayer_connector::MultiplayerConector};
use game_model::game::Game;

use game_render::{
    BoardRender, CustomRenderContext,
    constants::{
        FONT_SIZE, PATTERN_CELL_SIZE, PATTERN_COL_GAP, PATTERN_ELEMENT_GAP, PATTERN_PIECE_SIZE,
        PATTERN_ROW_GAP, TEXT_LINE_SPACING,
    },
    layout::{LayoutConstants, compute_layout},
    sprite::{Colour, SpriteRender},
};

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
        layout: LayoutConstants,
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
                        message.to_string(),
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
            render_context: CustomRenderContext::new(&layout),
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
                let layout: LayoutConstants = *(*self.board_render).borrow().get_layout();
                handle_player_input(
                    &mut self.game,
                    &mut self.command_handler,
                    &mut self.render_context,
                    canvas,
                    &layout,
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
        board_render.render(&game.board, &self.render_context, canvas);

        let layout: LayoutConstants = *board_render.get_layout();

        if self.render_context.show_patterns {
            draw_patterns(&self.render_context, &layout);
        } else {
            for (i, text) in description(&self.render_context, &game, &self.team_names)
                .iter()
                .enumerate()
            {
                let color: Colour = *board_render.get_team_color(game.current_team_index);
                draw_text(
                    text.as_str(),
                    layout.text_x,
                    layout.text_y + (i as f32) * FONT_SIZE * TEXT_LINE_SPACING,
                    FONT_SIZE,
                    color.into(),
                );
            }
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

    fn handle_resize(&mut self, new_width: f32, new_height: f32) {
        let new_layout = compute_layout(new_width, new_height);
        self.board_render.borrow_mut().set_layout(&new_layout);
        self.render_context.update_buttons(&new_layout);
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
    layout: &LayoutConstants,
) {
    if is_key_pressed(KeyCode::P) || render_context.button_patterns.clicked(canvas) {
        render_context.show_patterns = !render_context.show_patterns;
    } else if is_key_pressed(KeyCode::U) || render_context.button_undo.clicked(canvas) {
        let game_clone = (**game).borrow().clone();
        command_handler.handle_new_command(game_clone, &GameCommand::Undo);
    } else if is_key_pressed(KeyCode::G) {
        render_context.show_debug_overlay = !render_context.show_debug_overlay;
    } else if is_key_pressed(KeyCode::D) {
        if let Err(e) = export_to_file("exported_game", &command_handler.get_past_commands()) {
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
        let next_game_state = render_context.game_state.on_click(
            &layout.cell_hovered(canvas),
            game_clone,
            command_handler,
        );

        info!("{:?} -> {:?}", render_context.game_state, next_game_state);
        render_context.game_state = next_game_state;
    }
}

use game_model::{
    pattern::{Pattern, PatternComponent},
    piece::PieceKind,
};
use macroquad::texture::DrawTextureParams;

/// Draw the pattern infographic in the text area.
/// Called from render_internal when show_patterns is true.
fn draw_patterns(ctx: &CustomRenderContext, layout: &LayoutConstants) {
    let cols = 3;
    let start_x = layout.text_x;
    let start_y = layout.text_y - FONT_SIZE / 2.;

    let own_color = Color::from_rgba(60, 200, 60, 255);
    let free_color = WHITE;
    let any_color = Color::from_rgba(160, 160, 160, 255);

    let patterns = Pattern::all_patterns();

    // Precompute per-column max card widths so cards in each column are aligned
    let mut col_widths = vec![0.0f32; cols];
    for (i, p) in patterns.iter().enumerate() {
        let c = i % cols;
        let gc = p.components[0].len() as f32;
        let grid_w = gc * PATTERN_CELL_SIZE;
        let card_w = grid_w + PATTERN_ELEMENT_GAP * 3.0 + PATTERN_PIECE_SIZE;
        if card_w > col_widths[c] {
            col_widths[c] = card_w;
        }
    }

    // Compute column start positions
    let mut col_starts = vec![start_x; cols];
    for c in 1..cols {
        col_starts[c] = col_starts[c - 1] + col_widths[c - 1] + PATTERN_COL_GAP;
    }

    // Precompute row heights
    let mut row_heights: Vec<f32> = Vec::new();
    for (i, p) in patterns.iter().enumerate() {
        let r = i / cols;
        let gr = p.components.len() as f32;
        let card_h = gr * PATTERN_CELL_SIZE;
        if r >= row_heights.len() {
            row_heights.push(card_h);
        } else if card_h > row_heights[r] {
            row_heights[r] = card_h;
        }
    }

    // ── Legend row ──
    let legend_cell = PATTERN_CELL_SIZE * 0.6;
    let legend_label_size = FONT_SIZE * 0.5;
    let legend_gap = PATTERN_ELEMENT_GAP * 0.5;
    let mut lx = start_x;
    let ly = start_y;

    // Own piece: filled green
    draw_rectangle(lx, ly, legend_cell, legend_cell, own_color);
    lx += legend_cell + legend_gap;
    draw_text("own", lx, ly + legend_cell * 0.85, legend_label_size, WHITE);
    lx += legend_label_size * 3.0 + PATTERN_ELEMENT_GAP;

    // Free: filled white
    draw_rectangle(lx, ly, legend_cell, legend_cell, free_color);
    lx += legend_cell + legend_gap;
    draw_text(
        "free",
        lx,
        ly + legend_cell * 0.85,
        legend_label_size,
        WHITE,
    );
    lx += legend_label_size * 4.0 + PATTERN_ELEMENT_GAP;

    // Any: outlined gray
    draw_rectangle_lines(lx, ly, legend_cell, legend_cell, 2.0, any_color);
    lx += legend_cell + legend_gap;
    draw_text("any", lx, ly + legend_cell * 0.85, legend_label_size, WHITE);

    let legend_bottom = start_y + legend_cell + PATTERN_ELEMENT_GAP;

    for (i, pattern) in patterns.iter().enumerate() {
        let col = i % cols;
        let row = i / cols;

        let grid_rows = pattern.components.len() as f32;
        let grid_cols = pattern.components[0].len() as f32;
        let grid_w = grid_cols * PATTERN_CELL_SIZE;
        let grid_h = grid_rows * PATTERN_CELL_SIZE;

        let card_h = row_heights[row];
        let card_x = col_starts[col];
        let card_y = legend_bottom + row as f32 * (card_h + PATTERN_ROW_GAP);

        // Center grid vertically within card
        let grid_y = card_y + (card_h - grid_h) / 2.0;

        // Draw mini-grid
        for (gy, component_row) in pattern.components.iter().enumerate() {
            for (gx, component) in component_row.iter().enumerate() {
                let x = card_x + gx as f32 * PATTERN_CELL_SIZE;
                let y = grid_y + gy as f32 * PATTERN_CELL_SIZE;

                match component {
                    PatternComponent::OwnPiece => {
                        draw_rectangle(x, y, PATTERN_CELL_SIZE, PATTERN_CELL_SIZE, own_color);
                    }
                    PatternComponent::Free => {
                        draw_rectangle(x, y, PATTERN_CELL_SIZE, PATTERN_CELL_SIZE, free_color);
                    }
                    PatternComponent::Any => {
                        draw_rectangle_lines(
                            x,
                            y,
                            PATTERN_CELL_SIZE,
                            PATTERN_CELL_SIZE,
                            2.0,
                            any_color,
                        );
                    }
                }
                draw_rectangle_lines(
                    x,
                    y,
                    PATTERN_CELL_SIZE,
                    PATTERN_CELL_SIZE,
                    1.0,
                    Color::from_rgba(60, 60, 60, 160),
                );
            }
        }

        // Result piece sprite (no arrow glyph — not in font; wider gap for separation)
        let sprite_x = card_x + grid_w + PATTERN_ELEMENT_GAP * 3.0;
        let sprite_y = grid_y + (grid_h - PATTERN_PIECE_SIZE) / 2.0;

        let source_rect = SpriteRender::piece_sprite_rect(pattern.turn_into);
        let rotation = if pattern.turn_into == PieceKind::HorizontalBar {
            1.57
        } else {
            0.
        };
        draw_texture_ex(
            &ctx.pieces_texture,
            sprite_x,
            sprite_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(PATTERN_PIECE_SIZE, PATTERN_PIECE_SIZE)),
                source: Some(source_rect),
                rotation,
                ..Default::default()
            },
        );
    }
}

const EXPORTED_GAMES_DIR: &str = "game-core/tests/exported_games";

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
    file.write_all(format!("// {}\n", message).as_ref())?;
    file.write_all(content.serialize_json().as_bytes())?;

    Ok(())
}

#[cfg(target_family = "wasm")]
async fn post_error_report(url: String, message: String, content: Vec<GameCommand>) {
    use wasm_bindgen::JsValue;

    let request_init = web_sys::RequestInit::new();
    request_init.set_method("POST");
    request_init.set_mode(web_sys::RequestMode::Cors);

    let body = format!("// {}\n\n{}", message, content.serialize_json());
    request_init.set_body(&JsValue::from_str(&body));

    let request =
        match web_sys::Request::new_with_str_and_init(&(url + ":3030/error_report"), &request_init)
        {
            Ok(request) => request,
            Err(error) => {
                println!("{:?}", error);
                return;
            }
        };

    if let Err(error) = request
        .headers()
        .set("Content-Type", "text/plain;charset=UTF-8")
    {
        println!("{:?}", error);
        return;
    }

    let Some(window) = web_sys::window() else {
        println!("window unavailable while posting error report");
        return;
    };

    if let Err(error) =
        wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await
    {
        println!("{:?}", error);
    }
}

#[cfg(test)]
mod tests {}
