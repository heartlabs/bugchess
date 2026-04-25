use crate::{
    matchbox::MatchboxClient,
    states::{GameState, core_game_state::CoreGameState},
};
use egui_macroquad::{
    egui,
    egui::{
        Align, Color32, FontData, FontDefinitions, FontFamily, FontTweak, Layout, TextEdit, Visuals,
    },
};
use game_core::{core_game::CoreGameSubstate, multiplayer_connector::MultiplayerConector};

use game_model::{Point2, game::*};
use game_render::{
    BoardRender,
    constants::{BOARD_HEIGHT, BOARD_WIDTH, CELL_WIDTH, FONT_SIZE},
    layout::compute_layout,
    render_events::RenderEventConsumer,
};
use std::{
    cell::RefCell,
    fmt::{Display, Formatter},
    rc::Rc,
    sync::Arc,
};

use game_core::board_event_consumer::BoardEventConsumer;
use macroquad::prelude::*;
use macroquad_canvas::Canvas2D;

use game_core::{
    game_controller::GameCommand,
    game_events::{Event, PlayerAction},
};
use game_events::event_broker::EventBroker;

pub struct LoadingState {
    core_game_state: Option<CoreGameState>,
    sub_state: LoadingSubState,
    client: Option<MultiplayerConector>,
    room_id: String,
}

#[derive(Debug, Copy, Clone)]
pub enum LoadingSubState {
    Register,
    Matchmaking,
    JoinMatch,
    WaitForOpponent,
    GameMode,
    SetupGame,
}

impl Display for LoadingSubState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display_name = match self {
            LoadingSubState::Register => "Register",
            LoadingSubState::Matchmaking => "Matchmaking",
            LoadingSubState::JoinMatch => "Joining Match",
            LoadingSubState::WaitForOpponent => "Wait for Opponent",
            LoadingSubState::GameMode => "Choose Game Mode",
            LoadingSubState::SetupGame => "Set Up Game",
        };

        write!(f, "{}", display_name)
    }
}

impl LoadingState {
    pub fn new(canvas_width: f32, canvas_height: f32) -> Self {
        let game = Rc::new(RefCell::new(init_game()));
        let mut event_broker = EventBroker::new();
        event_broker.subscribe(Box::new(BoardEventConsumer::new(Rc::clone(&game))));

        let layout = compute_layout(canvas_width, canvas_height);
        let board_render = Rc::new(RefCell::new(BoardRender::new(&(*game).borrow(), &layout)));
        event_broker.subscribe(Box::new(RenderEventConsumer::new(&board_render)));

        let room_id = "standard_room".to_string();

        LoadingState {
            core_game_state: Option::Some(CoreGameState::new(
                game,
                event_broker,
                board_render,
                Option::None,
                false,
                vec!["Red".to_string(), "Yellow".to_string()],
                layout,
            )),
            sub_state: LoadingSubState::GameMode,
            client: Option::None,
            room_id,
        }
    }

    fn egui_select_room(&mut self, ui: &mut egui::Ui) {
        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(ui.available_rect_before_wrap())
                .layout(Layout::top_down_justified(Align::Center)),
        );
        child_ui.label("Enter Room ID");
        child_ui.add(
            TextEdit::singleline(&mut self.room_id)
                .desired_width(f32::INFINITY)
                .text_color(Color32::from_rgb(0, 200, 0)),
        );
        if child_ui.button("OK").clicked() {
            self.join_room(&self.room_id.clone());
        }
    }

    pub fn join_room(&mut self, room_id: &str) {
        let client = MatchboxClient::new_connector(room_id);
        self.client = Some(client);
        self.core_game_state.as_mut().unwrap().is_multi_player = true;

        self.sub_state = LoadingSubState::Matchmaking;
    }

    #[cfg(target_family = "wasm")]
    pub fn offline_game(&mut self) {
        self.sub_state = LoadingSubState::SetupGame;
        self.core_game_state.as_mut().unwrap().is_multi_player = false;
    }

    fn select_game_mode(&mut self) {
        egui_macroquad::ui(|egui_ctx| {
            egui_setup_fonts(egui_ctx);

            egui::CentralPanel::default().show(egui_ctx, |ui| {
                // Center a fixed-size child UI for the menu
                let menu_width = 400.0;
                let menu_height = 220.0;
                let center = ui.max_rect().center();
                let menu_rect =
                    egui::Rect::from_center_size(center, egui::vec2(menu_width, menu_height));
                let mut child_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(menu_rect)
                        .layout(Layout::top_down(Align::Center)),
                );

                // Title with smaller font and spacing
                child_ui.add_space(10.0);
                child_ui.label(egui::RichText::new("Select game mode").heading().size(32.0));
                child_ui.add_space(30.0);

                if child_ui
                    .add_sized(
                        [menu_width * 0.85, 56.0],
                        egui::Button::new(egui::RichText::new("Offline").size(24.0)),
                    )
                    .clicked()
                {
                    self.sub_state = LoadingSubState::SetupGame;
                }
                child_ui.add_space(10.0);
                if child_ui
                    .add_sized(
                        [menu_width * 0.85, 56.0],
                        egui::Button::new(egui::RichText::new("Online").size(24.0)),
                    )
                    .clicked()
                {
                    self.core_game_state.as_mut().unwrap().is_multi_player = true;
                    self.sub_state = LoadingSubState::Register;
                }
            });
        });
    }

    fn select_room(&mut self) {
        egui_macroquad::ui(|egui_ctx| {
            egui_setup_fonts(egui_ctx);

            egui::CentralPanel::default()
                //.fixed_size(egui::Vec2::new(800., 600.))
                //.fixed_rect(emath::Rect::from_two_pos(emath::pos2(0., 0.), emath::pos2(1200., 1000.)))
                //.resizable(false)
                //.collapsible(false)
                .show(egui_ctx, |ui| {
                    //ui.set_width(ui.max_rect().width());
                    self.egui_select_room(ui);
                });
        });
    }
}

impl GameState for LoadingState {
    fn update(&mut self, _canvas: &Canvas2D) -> Option<Box<dyn GameState>> {
        match &self.sub_state {
            LoadingSubState::GameMode => {
                self.select_game_mode();
            }
            LoadingSubState::Register => {
                self.select_room();
            }
            LoadingSubState::Matchmaking => {
                let client = self.client.as_mut().unwrap();
                client.matchmaking();

                if client.is_ready() {
                    self.sub_state = LoadingSubState::JoinMatch;
                }
            }
            LoadingSubState::JoinMatch => {
                let core_game_state = self.core_game_state.as_mut().unwrap();

                let mut matchbox_client = self.client.take().unwrap();

                matchbox_client.signal_connect();

                let multiplayer_events = Option::Some(Rc::new(RefCell::new(matchbox_client)));
                core_game_state.command_handler.multiplayer_connector =
                    Some(Rc::clone(multiplayer_events.as_ref().unwrap()));

                core_game_state.matchbox_events = multiplayer_events;

                self.sub_state = LoadingSubState::WaitForOpponent;
            }
            LoadingSubState::WaitForOpponent => {
                let core_game_state = self.core_game_state.as_mut().unwrap();

                let (events, own_player_id) = {
                    let mut client = core_game_state
                        .matchbox_events
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .borrow_mut();
                    (client.try_recieve(), client.get_own_player_id().unwrap())
                };

                let opponent_index = events
                    .iter()
                    .filter_map(|e| match &e.event {
                        Event::PlayerAction(PlayerAction::Connect(_, i)) => Some((i, i == &1)),
                        Event::PlayerAction(PlayerAction::NewGame((p1, _p2))) => {
                            if p1 == &own_player_id {
                                Some((&1, false))
                            } else {
                                Some((&0, false))
                            }
                        }
                        _ => None,
                    })
                    .next();

                if let Some((opponent_index, initiator)) = opponent_index {
                    {
                        debug!("opponent_index {}, initiator {}", opponent_index, initiator);
                        let mut client = core_game_state
                            .matchbox_events
                            .as_ref()
                            .unwrap()
                            .as_ref()
                            .borrow_mut();

                        if opponent_index == &1 {
                            // The opponent says he is second - so we must be first
                            client.override_own_player_index = Some(0); // we will always be this index - so lock it
                        } else {
                            client.override_own_player_index = Some(1);
                        }

                        if initiator {
                            client.signal_new_game();
                        }
                    }

                    if initiator {
                        let num_teams = 2;
                        let set_up_actions =
                            set_up_pieces(num_teams, &(*core_game_state.game).borrow());
                        for start_event in &set_up_actions {
                            core_game_state
                                .command_handler
                                .handle_new_command(core_game_state.game_clone(), start_event);
                        }
                    } else {
                        core_game_state.set_sub_state(CoreGameSubstate::Wait);
                    }

                    events
                        .iter()
                        .filter(|e| matches!(e.event, Event::GameCommand(_)))
                        .for_each(|e| {
                            core_game_state
                                .command_handler
                                .handle_remote_command(core_game_state.game_clone(), e)
                        });
                } else {
                    debug!("waiting for opponent message");
                    return None;
                }

                return Option::Some(Box::new(self.core_game_state.take().unwrap()));
            }

            LoadingSubState::SetupGame => {
                let core_game_state = self.core_game_state.as_mut().unwrap();
                let num_teams = 2;
                let set_up_actions = set_up_pieces(num_teams, &(*core_game_state.game).borrow());
                for start_event in &set_up_actions {
                    core_game_state
                        .command_handler
                        .handle_new_command(core_game_state.game_clone(), start_event);
                }
                return Option::Some(Box::new(self.core_game_state.take().unwrap()));
            }
        }

        Option::None
    }

    fn render(&self, _canvas: &Canvas2D) {
        if self.core_game_state.is_some() {
            draw_text(
                &format!("Loading: {}... ", self.sub_state),
                10.,
                CELL_WIDTH * BOARD_WIDTH as f32 / 2.,
                FONT_SIZE * 1.5,
                GREEN,
            );
        }
    }

    fn uses_egui(&self) -> bool {
        matches!(
            self.sub_state,
            LoadingSubState::Register | LoadingSubState::GameMode
        )
    }
}

fn egui_setup_fonts(egui_ctx: &egui::Context) {
    let mut font_definitions = FontDefinitions::default();
    let mut font_data =
        FontData::from_static(include_bytes!("../../resources/fonts/Koulen-Regular.ttf"));
    font_data.tweak = FontTweak::default();
    font_data.tweak.scale = 1.;
    font_definitions
        .font_data
        .insert("bugchess".to_owned(), Arc::new(font_data));
    // Put my font first (highest priority):
    font_definitions
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "bugchess".to_owned());
    egui_ctx.set_fonts(font_definitions);
    let visuals = Visuals {
        collapsing_header_frame: true,
        ..Default::default()
    };
    egui_ctx.set_visuals(visuals);
}

fn set_up_pieces(team_count: usize, _game_ref: &Game) -> Vec<GameCommand> {
    let start_pieces = 6;

    let mut events = vec![];

    for _ in 0..team_count {
        events.push(GameCommand::InitPlayer(start_pieces));
    }

    for team_id in 0..team_count {
        let target_point = Point2::new((2 + team_id * 3) as u8, (2 + team_id * 3) as u8);
        events.push(GameCommand::PlacePiece(target_point));
        events.push(GameCommand::NextTurn);

        /*let mut piece = Piece::new(team_id, PieceKind::Simple);
        piece.exhaustion.reset();
        finish_turn.place_piece(target_point, piece);

        for _ in 0..start_pieces {
            finish_turn.add_unused_piece(team_id);
        }

        let compound_event = finish_turn.build();
        BoardEventConsumer::flush_unsafe(&mut game_ref.clone(), &compound_event);
        events.push(compound_event);*/
    }

    events
}

fn init_game() -> Game {
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

    Game::new(teams, BOARD_WIDTH, BOARD_HEIGHT)
}
