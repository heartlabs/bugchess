use std::{cell::RefCell, rc::Rc};

use game_core::{
    core_game::CoreGameSubstate, event_broker::EventBroker, game_controller::GameController,multiplayer_connector::MultiplayerConector, 
};
use game_model::game::Game;

use game_render::{constants::cell_hovered, BoardRender, CustomRenderContext};

use crate::{constants::ONLINE, GameState};
use macroquad::prelude::*;
use macroquad_canvas::Canvas2D;

pub struct CoreGameState {
    pub game: Rc<RefCell<Game>>,
    pub(crate) event_broker: EventBroker,
    board_render: Rc<RefCell<BoardRender>>,
    pub matchbox_events: Option<Rc<RefCell<MultiplayerConector>>>,
    render_context: CustomRenderContext,
    own_player_team_id: Option<usize>,
}

impl CoreGameState {
    pub(crate) fn new(
        game: Rc<RefCell<Game>>,
        event_broker: EventBroker,
        board_render: Rc<RefCell<BoardRender>>,
        matchbox_events: Option<Rc<RefCell<MultiplayerConector>>>,
    ) -> Self {
        CoreGameState {
            game,
            event_broker,
            board_render,
            matchbox_events,
            render_context: CustomRenderContext::new(),
            own_player_team_id: Option::None,
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

            if self.own_player_team_id.is_none() {
                self.own_player_team_id = (**self.matchbox_events.as_ref().unwrap())
                    .borrow()
                    .get_own_player_index();
                info!("own player id: {:?}", self.own_player_team_id);
            }
        }

        match self.render_context.game_state {
            CoreGameSubstate::Wait => {
                if can_control_player(&(*self.game).borrow(), &mut self.own_player_team_id, ONLINE)
                {
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

        check_if_somebody_won(&(*self.game).borrow(), &mut self.render_context);

        if false {
            self.board_render = Rc::new(RefCell::new(BoardRender::new(&(*self.game).borrow())));
        }

        (*self.board_render).borrow_mut().update();

        Option::None
    }

    fn render(&self, canvas: &Canvas2D) {
        let board_render = (*self.board_render).borrow();
        let game = (*self.game).borrow();
        board_render.render(&(*self.game).borrow().board, &self.render_context, canvas);

        for (i, text) in description(&self.render_context, &game).iter().enumerate() {
            draw_text(
                text.as_str(),
                10.,
                670. + (i * 50) as f32,
                50.,
                board_render.get_team_color(game.current_team_index).clone(),
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
    game: &mut Rc<RefCell<Game>>,
    event_broker: &mut EventBroker,
    render_context: &mut CustomRenderContext,
    canvas: &Canvas2D,
) {
    if is_key_pressed(KeyCode::U) || render_context.button_undo.clicked(canvas) {
        event_broker.undo();
    } else if is_key_pressed(KeyCode::Enter)
        || is_key_pressed(KeyCode::KpEnter)
        || render_context.button_next.clicked(canvas)
    {
        let event_option = GameController::next_turn(&(**game).borrow());
        event_broker.handle_new_event(&event_option);

        render_context.game_state = CoreGameSubstate::Wait;
        // BoardEventConsumer::flush_unsafe(game.as_ref().borrow_mut().borrow_mut(), &event_option);
    } else if is_mouse_button_pressed(MouseButton::Left) {
        let builder_option = None;
        let game_clone = (**game).borrow().clone();
        let next_game_state =
            render_context
                .game_state
                .on_click(&cell_hovered(canvas), game_clone, event_broker);
        //.on_click(&cell_hovered(canvas), game, &mut builder_option);

        info!("{:?} -> {:?}", render_context.game_state, next_game_state);
        render_context.game_state = next_game_state;

        if let Some(game_action) = builder_option {
            event_broker.handle_new_event(&game_action);
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::test_utils::*;
    use game_core::core_game::CoreGameSubstate;
    use game_model::{
        game::Game,
        piece::{EffectKind::Protection, PieceKind},
    };

    #[test]
    fn test_merge_piece_multiplayer() {
        let (mut test_game1, mut test_game2) = create_multiplayer_game();

        // Make Move
        let game_state1 = CoreGameSubstate::Place;
        let game_state1 = test_game1.click_at_pos((1, 1), game_state1);
        let game_state1 = test_game1.click_at_pos((1, 2), game_state1);
        let game_state1 = test_game1.click_at_pos((1, 3), game_state1);

        // Recieve move in Game 2
        test_game2.recieve_multiplayer_events();

        // Assertions Game 1
        assert_eq!(game_state1, CoreGameSubstate::Place);

        let game = &mut (*test_game1.game.borrow_mut());
        assert_eq!(game.board.placed_pieces(0).len(), 1);
        assert!(game.board.placed_pieces(1).is_empty());

        assert_piece_at(game, (1, 2), PieceKind::VerticalBar);

        // Assertions Game 2
        let game = &(*test_game2.game).borrow();
        assert_eq!(game.board.placed_pieces(0).len(), 1);
        assert!(game.board.placed_pieces(1).is_empty());

        assert_piece_at(game, (1, 2), PieceKind::VerticalBar);
    }

    #[test]
    fn test_place_single_piece_multiplayer() {
        let (mut test_game1, mut test_game2) = create_multiplayer_game();

        // Make Move
        let game_clone = (*test_game1.game.borrow()).clone();
        let game_state1 = CoreGameSubstate::Place.on_click(
            &(0, 0).into(),
            game_clone,
            &mut test_game1.event_broker,
        );

        // Recieve move in Game 2
        test_game2.recieve_multiplayer_events();

        // Assertions Game 1
        assert_eq!(game_state1, CoreGameSubstate::Place);

        let game = &(*test_game1.game.borrow());
        assert!(game.board.placed_pieces(0).len() == 1);
        assert!(game.board.placed_pieces(1).is_empty());

        assert_piece_at(&game, (0, 0), PieceKind::Simple);

        // Assertions Game 2
        let game = &(*test_game2.game).borrow();
        assert!(game.board.placed_pieces(0).len() == 1);
        assert!(game.board.placed_pieces(1).is_empty());

        assert_piece_at(&game, (0, 0), PieceKind::Simple);
    }

    #[test]
    fn test_remove_effects() {
        let (mut test_game1, mut test_game2) = create_multiplayer_game();

        // skip a turn each to have enough unused pieces
        test_game1.next_turn();
        test_game2.recieve_multiplayer_events();
        test_game2.next_turn();
        test_game1.recieve_multiplayer_events();

        // Make Move
        let game_state1 = CoreGameSubstate::Place;
        let game_state1 = test_game1.click_at_pos((1, 0), game_state1);
        let game_state1 = test_game1.click_at_pos((0, 1), game_state1);
        let game_state1 = test_game1.click_at_pos((2, 1), game_state1);
        let game_state1 = test_game1.click_at_pos((1, 2), game_state1);

        test_game1.next_turn();

        // Recieve move in Game 2
        test_game2.recieve_multiplayer_events();

        // Assertions Game 1
        assert_eq!(game_state1, CoreGameSubstate::Place);

        {
            let game = &(*test_game1.game.borrow());
            println!("Board:\n{}", game.board);
            assert_eq!(game.board.placed_pieces(0).len(), 1);
            assert!(game.board.placed_pieces(1).is_empty());

            assert_piece_at(game, (1, 1), PieceKind::Castle);
            assert_protection_at(game, (0, 0));
            assert_protection_at(game, (0, 1));
            assert_protection_at(game, (0, 2));
            assert_protection_at(game, (1, 0));
            assert_protection_at(game, (1, 1));
            assert_protection_at(game, (1, 2));
            assert_protection_at(game, (2, 0));
            assert_protection_at(game, (2, 1));
            assert_protection_at(game, (2, 2));
        }

        // Prepare attack of castle
        test_game2.click_at_pos((4, 0), CoreGameSubstate::Place);
        test_game2.click_at_pos((4, 1), CoreGameSubstate::Place);
        test_game2.click_at_pos((3, 1), CoreGameSubstate::Place);
        test_game2.click_at_pos((5, 1), CoreGameSubstate::Place);
        test_game2.click_at_pos((4, 2), CoreGameSubstate::Place);

        test_game2.next_turn();
        test_game1.recieve_multiplayer_events();
        test_game2.next_turn();

        // attack castle
        let game_state2 = test_game2.click_at_pos((4, 1), CoreGameSubstate::Place);
        test_game2.click_at_pos((1, 1), game_state2);

        let game = &(*test_game2.game.borrow());
        assert_eq!(game.board.placed_pieces(1).len(), 1);
        assert!(game.board.placed_pieces(0).is_empty());

        game.board.for_each_cell(|c| assert!(c.effects.is_empty()));
    }

    fn assert_protection_at(game: &Game, pos: (u8, u8)) {
        assert!(game.board.has_effect_at(&Protection, &pos.into()));
    }
    fn assert_piece_at(game: &Game, piece_pos: (u8, u8), piece_kind: PieceKind) {
        let placed_piece = game
            .board
            .get_piece_at(&piece_pos.into())
            .expect("Placed piece not found on board");
        assert_eq!(placed_piece.piece_kind, piece_kind);
    }
}
