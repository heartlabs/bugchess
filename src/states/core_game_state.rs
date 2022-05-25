use crate::{
    game_events::{CompoundEventType, EventBroker},
    GameEvent::{ChangeExhaustion, Remove},
    *,
};

use crate::{
    game_events::{EventComposer, EventConsumer},
    game_logic::game::Game,
    matchbox::MatchboxClient,
    GameEvent::Place,
    Power::{Blast, TargetedShoot},
};
use macroquad::prelude::*;

pub struct CoreGameState {
    game: Rc<RefCell<Box<Game>>>,
    pub(crate) event_broker: EventBroker,
    pub(crate) event_composer: EventComposer,
    board_render: Rc<RefCell<Box<BoardRender>>>,
    pub matchbox_events: Option<Rc<RefCell<Box<MatchboxClient>>>>,
    render_context: CustomRenderContext,
    own_player_id: Option<usize>,
}

impl CoreGameState {
    pub(crate) fn new(
        game: Rc<RefCell<Box<Game>>>,
        event_broker: EventBroker,
        event_composer: EventComposer,
        board_render: Rc<RefCell<Box<BoardRender>>>,
        matchbox_events: Option<Rc<RefCell<Box<MatchboxClient>>>>,
    ) -> Self {
        CoreGameState {
            game,
            event_broker,
            event_composer,
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

        for (i, text) in description(&self.render_context, (*self.game).borrow().as_ref())
            .iter()
            .enumerate()
        {
            draw_text(
                text.as_str(),
                10.,
                670. + (i * 50) as f32,
                50.,
                (*self.game).borrow().as_ref().current_team().color,
            );
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
                    &mut self.event_composer,
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
        (*self.board_render).borrow().as_ref().render(
            &(*self.game).borrow().as_ref().board,
            &self.render_context,
            canvas,
        );
    }
}

#[derive(Debug, Copy, Clone)]
pub enum CoreGameSubstate {
    Place,
    Move(Point2),
    Activate(Point2),
    Won(usize),
    Wait,
}

impl CoreGameSubstate {
    pub(crate) fn on_click(
        &self,
        target_point: &Point2,
        game: &mut Game,
        event_composer: &mut EventComposer,
    ) -> CoreGameSubstate {
        let board = &game.board;
        if !board.has_cell(target_point) {
            return CoreGameSubstate::Place;
        }

        match self {
            CoreGameSubstate::Place => {
                if let Some(target_piece) = board.get_piece_at(target_point) {
                    if target_piece.team_id == game.current_team_index {
                        if target_piece.can_move() {
                            return CoreGameSubstate::Move(*target_point);
                        } else if target_piece.can_use_special() {
                            return CoreGameSubstate::Activate(*target_point);
                        }
                    }
                } else if game.unused_piece_available() {
                    let new_piece = Piece::new(game.current_team_index, PieceKind::Simple);
                    event_composer.init_new_transaction(
                        vec![
                            GameEvent::Place(
                                *target_point,
                                new_piece,
                            ),
                            GameEvent::RemoveUnusedPiece(game.current_team_index),
                        ],
                        CompoundEventType::Place,
                    );

                    Self::push_effects_if_present(event_composer, &board, &new_piece, target_point);
                }
            }
            CoreGameSubstate::Move(itself) => {
                if let Some(target_piece) = board.get_piece_at(target_point) {
                    if *itself == *target_point && target_piece.can_use_special() {
                        if let Some(activatable) = target_piece.activatable {
                            return match activatable.kind {
                                Power::Blast => {
                                    let mut exhaustion_clone = target_piece.exhaustion.clone();
                                    exhaustion_clone.on_attack();
                                    event_composer.start_transaction(CompoundEventType::Attack(target_piece.piece_kind));
                                    for point in activatable.range.reachable_points(
                                        target_point,
                                        board,
                                        &RangeContext::Special(*target_piece),
                                    ) {
                                        if let Some(piece) = board.get_piece_at(&point) {
                                            event_composer.push_event(Remove(point, *piece));
                                            Self::remove_effects_if_present(event_composer, board, piece,&point);
                                        }
                                    }

                                    event_composer.push_event(ChangeExhaustion(target_piece.exhaustion, exhaustion_clone, *target_point));

                                    CoreGameSubstate::Place
                                }
                                Power::TargetedShoot => CoreGameSubstate::Activate(*target_point),
                            };
                        }
                    }
                    if target_piece.team_id == game.current_team_index && target_piece.can_move() {
                        return CoreGameSubstate::Move(*target_point);
                    }
                }

                let selected_piece = board.get_piece_at(&itself).unwrap();
                if let Some(m) = selected_piece.movement.as_ref() {
                    if m.range
                        .reachable_points(&itself, board, &RangeContext::Moving(*selected_piece))
                        .contains(target_point)
                    {
                        event_composer.start_transaction(CompoundEventType::Move);

                        if let Some(target_piece) = board.get_piece_at(target_point) {
                            event_composer.push_event(Remove(*target_point, *target_piece));
                        }

                        event_composer.push_event(Remove(*itself, *selected_piece));
                        event_composer.push_event(Place(*target_point, *selected_piece));

                        if let Some(target_piece) = board.get_piece_at(target_point) {
                            Self::remove_effects_if_present(event_composer, board, target_piece, target_point);
                        }

                        let mut exhaustion_clone = selected_piece.exhaustion.clone();
                        exhaustion_clone.on_attack();

                        event_composer.push_event(ChangeExhaustion(selected_piece.exhaustion, exhaustion_clone, *target_point));
                    }
                }
            }
            CoreGameSubstate::Activate(active_piece_pos) => {
                let active_piece = board.get_piece_at(active_piece_pos).unwrap();
                if let Some(target_piece) = board.get_piece_at(target_point) {
                    if target_piece.team_id != game.current_team_index
                        && active_piece.can_use_special()
                    {
                        let mut exhaustion_clone = target_piece.exhaustion.clone();
                        exhaustion_clone.on_attack();

                        event_composer.init_new_transaction(
                            vec![
                                Remove(*target_point, *target_piece),
                                ChangeExhaustion(target_piece.exhaustion, exhaustion_clone, *active_piece_pos),
                            ],
                            CompoundEventType::Attack(active_piece.piece_kind),
                        );

                        Self::remove_effects_if_present(event_composer, board, target_piece, target_point);
                    }
                }
            }
            CoreGameSubstate::Won(team) => {
                return CoreGameSubstate::Won(*team);
            }
            CoreGameSubstate::Wait => return CoreGameSubstate::Wait,
        }

        CoreGameSubstate::Place
    }

    fn push_effects_if_present(event_composer: &mut EventComposer, board: &Board, new_piece: &Piece, pos: &Point2) {
        if let Some(effect) = new_piece.effect {
            effect
                .range
                .reachable_points(pos, &board, &RangeContext::Area)
                .iter()
                .for_each(|&point| event_composer.push_event(GameEvent::AddEffect(EffectKind::Protection, point)));
        }
    }

    fn remove_effects_if_present(event_composer: &mut EventComposer, board: &Board, piece: &Piece, pos: &Point2) {
        if let Some(effect) = piece.effect {
            effect
                .range
                .reachable_points(pos, &board, &RangeContext::Area)
                .iter()
                .for_each(|&point| event_composer.push_event(GameEvent::RemoveEffect(EffectKind::Protection, point)));
        }
    }

    pub(crate) fn merge_patterns(board: &mut Board, event_composer: &mut EventComposer) {
        //let mut remove_pieces = vec![];
        //let mut place_pieces = vec![];

        for pattern in &Pattern::all_patterns() {
            for x in 0..board.w as usize - pattern.components[0].len() + 1 {
                for y in 0..board.h as usize - pattern.components.len() + 1 {
                    let matched = { board.match_pattern(&pattern, x as u8, y as u8) };

                    if let Some(mut matched_entities) = matched {
                        let any_team_id = board.get_piece_at(&matched_entities[0]).unwrap().team_id;
                        println!("Pattern matched!");
                        if matched_entities
                            .iter()
                            .map(|point| board.get_piece_at(point).unwrap())
                            .all(|piece| piece.team_id == any_team_id && !piece.dying)
                        {
                            matched_entities.iter_mut().for_each(|point| {
                                // println!("Going to remove matched piece {:?}", matched_piece);
                                {
                                    let matched_piece = board.get_piece_mut_at(point).unwrap();
                                    event_composer
                                        .push_event(GameEvent::Remove(*point, *matched_piece));
                                    matched_piece.dying = true;
                                }
                                let matched_piece = board.get_piece_at(point).unwrap();

                                Self::remove_effects_if_present(event_composer, board, matched_piece, point);

                            });

                            let new_piece = Piece::new(any_team_id, pattern.turn_into);

                            let new_piece_x = x as u8 + pattern.new_piece_relative_position.x;
                            let new_piece_y = y as u8 + pattern.new_piece_relative_position.y;

                            let new_piece_pos = Point2::new(new_piece_x, new_piece_y);
                            event_composer.push_event(GameEvent::Place(
                                new_piece_pos,
                                new_piece,
                            ));

                            Self::push_effects_if_present(event_composer, &board, &new_piece, &new_piece_pos);

                            /* println!(
                                "Matched pattern at {}:{}; new piece at {}:{}",
                                x, y, new_piece_x, new_piece_y
                            );*/
                        }
                    }
                }
            }
        }
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
    mut event_broker: &mut EventBroker,
    mut event_composer: &mut EventComposer,
    render_context: &mut CustomRenderContext,
    canvas: &Canvas2D,
) {
    if is_mouse_button_pressed(MouseButton::Left) {
        let next_game_state = render_context.game_state.on_click(
            &cell_hovered(canvas),
            game.as_ref().borrow_mut().borrow_mut(),
            &mut event_composer,
        );
        info!("{:?} -> {:?}", render_context.game_state, next_game_state);
        render_context.game_state = next_game_state;

        event_composer.flush();
        CoreGameSubstate::merge_patterns(
            &mut (**game).borrow_mut().as_mut().board,
            &mut event_composer,
        );
        event_composer.commit();
    }

    if is_key_pressed(KeyCode::Enter)
        || is_key_pressed(KeyCode::KpEnter)
        || render_context.button_next.clicked(canvas)
    {
        next_turn(&mut game, &mut event_composer);
        render_context.game_state = CoreGameSubstate::Wait;
    }

    if is_key_pressed(KeyCode::U) || render_context.button_undo.clicked(canvas) {
        event_broker.undo();
    }

    event_composer.flush();
    event_composer
        .drain_commits()
        .iter()
        .for_each(|e| event_broker.handle_new_event(e));
}

fn next_turn(game: &mut Rc<RefCell<Box<Game>>>, event_composer: &mut EventComposer) {
    {
        let mut g = (**game).borrow_mut();
        let current_team_index = g.current_team_index;
        event_composer.start_transaction(CompoundEventType::FinishTurn);
        event_composer.push_event(GameEvent::NextTurn);
        event_composer.push_event(GameEvent::AddUnusedPiece(current_team_index));
        event_composer.push_event(GameEvent::AddUnusedPiece(current_team_index));

        g.board
            .for_each_placed_piece(|point, piece| {
                if piece.movement.is_none() && piece.activatable.is_none() {
                    return;
                }

                let mut exhaustion_clone = piece.exhaustion.clone();
                exhaustion_clone.reset();

                if exhaustion_clone != piece.exhaustion {
                    event_composer.push_event(GameEvent::ChangeExhaustion(piece.exhaustion, exhaustion_clone, point));
                }
            });
    }
    event_composer.commit();
}
