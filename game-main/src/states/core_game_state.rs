use std::{borrow::BorrowMut, cell::RefCell, rc::Rc};

use game_events::{
    actions::compound_events::GameAction,
    board_event_consumer::BoardEventConsumer, 
    event_broker::EventBroker, 
    core_game::CoreGameSubstate,
};
use game_logic::{
    game::Game,
};
use macroquad_canvas::Canvas2D;

use crate::{
    constants::{cell_hovered, ONLINE},
    matchbox::MatchboxClient,
    rendering::{BoardRender, CustomRenderContext},
    GameState,
};
use macroquad::prelude::*;

pub struct CoreGameState {
    pub game: Rc<RefCell<Box<Game>>>,
    pub(crate) event_broker: EventBroker,
    board_render: Rc<RefCell<Box<BoardRender>>>,
    pub matchbox_events: Option<Rc<RefCell<Box<MatchboxClient>>>>,
    render_context: CustomRenderContext,
    own_player_id: Option<usize>,
}

impl CoreGameState {
    pub(crate) fn new(
        game: Rc<RefCell<Box<Game>>>,
        event_broker: EventBroker,
        board_render: Rc<RefCell<Box<BoardRender>>>,
        matchbox_events: Option<Rc<RefCell<Box<MatchboxClient>>>>,
    ) -> Self {
        CoreGameState {
            game,
            event_broker,
            board_render,
            matchbox_events,
            render_context: CustomRenderContext::new(),
            own_player_id: Option::None,
        }
    }

    pub fn set_sub_state(&mut self, sub_state: CoreGameSubstate) {
        self.render_context.game_state = sub_state;
    }
}

impl GameState for CoreGameState {
    fn update(&mut self, canvas: &Canvas2D) -> Option<Box<dyn GameState>> {
        if ONLINE {
            let recieved_events = (**self.matchbox_events.as_mut().unwrap())
                .borrow_mut()
                .try_recieve();

            recieved_events
                .iter()
                .for_each(|e| self.event_broker.handle_remote_event(&e));

            if self.own_player_id.is_none() {
                self.own_player_id = (**self.matchbox_events.as_ref().unwrap())
                    .borrow()
                    .as_ref()
                    .get_own_player_index();
                info!("own player id: {:?}", self.own_player_id);
            }
        }

        match self.render_context.game_state {
            CoreGameSubstate::Wait => {
                if can_control_player(
                    (*self.game).borrow().as_ref(),
                    &mut self.own_player_id,
                    ONLINE,
                ) {
                    self.render_context.game_state = CoreGameSubstate::Place;
                }
            }
            CoreGameSubstate::Won(_) => {}
            _ => {
                handle_player_input(
                    &mut self.game,
                    &mut self.event_broker,
                    &mut self.render_context,
                    canvas,
                );
            }
        }

        check_if_somebody_won((*self.game).borrow().as_ref(), &mut self.render_context);

        if false {
            self.board_render = Rc::new(RefCell::new(Box::new(BoardRender::new(
                (*self.game).borrow().as_ref(),
            ))));
        }

        (*self.board_render).borrow_mut().as_mut().update();

        Option::None
    }

    fn render(&self, canvas: &Canvas2D) {
        let board_render = (*self.board_render).borrow();
        let game = (*self.game).borrow();
        board_render.render(
            &(*self.game).borrow().as_ref().board,
            &self.render_context,
            canvas,
        );

        for (i, text) in description(&self.render_context, &game)
            .iter()
            .enumerate()
        {
            draw_text(
                text.as_str(),
                10.,
                670. + (i * 50) as f32,
                50.,
                board_render.team_colors[game.current_team_index],
            );
        }
    }

    fn uses_egui(&self) -> bool {
        false
    }
}


fn check_if_somebody_won(game: &Game, render_context: &mut CustomRenderContext) {
    let board = &game.board;
    if board.placed_pieces(0).is_empty() || game.num_unused_pieces_of(1) >= 20 {
        info!("Team 1 won");
        render_context.game_state = CoreGameSubstate::Won(1);
    }
    if board.placed_pieces(1).is_empty() || game.num_unused_pieces_of(0) >= 20 {
        info!("Team 0 won");
        render_context.game_state = CoreGameSubstate::Won(0);
    }
}

fn description(render_context: &CustomRenderContext, game: &Game) -> Vec<String> {
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
                format!("The {} team won", game.teams[team].name)
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

fn can_control_player(game: &Game, own_player_id: &mut Option<usize>, is_online: bool) -> bool {
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
    mut game: &mut Rc<RefCell<Box<Game>>>,
    event_broker: &mut EventBroker,
    render_context: &mut CustomRenderContext,
    canvas: &Canvas2D,
) {
    let mut event_option: Option<GameAction> = None;

    if is_key_pressed(KeyCode::U) || render_context.button_undo.clicked(canvas) {
        event_broker.undo();
    } else if is_key_pressed(KeyCode::Enter)
        || is_key_pressed(KeyCode::KpEnter)
        || render_context.button_next.clicked(canvas)
    {
        event_option = Some(next_turn(&mut game));
        render_context.game_state = CoreGameSubstate::Wait;
    } else if is_mouse_button_pressed(MouseButton::Left) {
        let next_game_state = render_context.game_state.on_click(
            &cell_hovered(canvas),
            game.as_ref().borrow_mut().borrow_mut(),
            &mut event_option,
        );
        info!("{:?} -> {:?}", render_context.game_state, next_game_state);
        render_context.game_state = next_game_state;

        if let Some(event_composer) = event_option.as_mut() {
            BoardEventConsumer::flush(game.as_ref().borrow_mut().borrow_mut(), event_composer);
            CoreGameSubstate::merge_patterns(
                &mut (**game).borrow_mut().as_mut().board,
                event_composer,
            );
            BoardEventConsumer::flush(game.as_ref().borrow_mut().borrow_mut(), event_composer);
        }
    }

    if let Some(compound_event) = event_option.as_mut() {
        BoardEventConsumer::flush(game.as_ref().borrow_mut().borrow_mut(), compound_event);
        event_broker.handle_new_event(compound_event)
    }
}

fn next_turn(game: &mut Rc<RefCell<Box<Game>>>) -> GameAction {
    let mut finish_turn = GameAction::finish_turn();
    {
        let g = (**game).borrow_mut();
        let current_team_index = g.current_team_index;

        finish_turn
            .add_unused_piece(current_team_index)
            .add_unused_piece(current_team_index);

        g.board.for_each_placed_piece(|point, piece| {
            if piece.movement.is_none() && piece.activatable.is_none() {
                return;
            }

            let mut exhaustion_clone = piece.exhaustion.clone();
            exhaustion_clone.reset();

            if exhaustion_clone != piece.exhaustion {
                finish_turn.change_exhaustion(piece.exhaustion, exhaustion_clone, point);
            }
        });
    }
    let mut finish_turn_action = finish_turn.build();

    BoardEventConsumer::flush(
        game.as_ref().borrow_mut().borrow_mut(),
        &mut finish_turn_action,
    );

    finish_turn_action
}
