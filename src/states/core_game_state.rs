use crate::{
    game_events::{CompoundEventType, EventBroker},
    GameEvent::{CompoundEvent, Exhaust, Remove},
    *,
};

use crate::matchbox::MatchboxClient;
use macroquad::prelude::*;

pub struct CoreGameState {
    board: Rc<RefCell<Box<Board>>>,
    pub(crate) event_broker: EventBroker,
    board_render: Rc<RefCell<Box<BoardRender>>>,
    pub matchbox_events: Option<Rc<RefCell<Box<MatchboxClient>>>>,
    render_context: CustomRenderContext,
    own_player_id: Option<usize>,
}

impl CoreGameState {
    pub(crate) fn new(
        board: Rc<RefCell<Box<Board>>>,
        event_broker: EventBroker,
        board_render: Rc<RefCell<Box<BoardRender>>>,
        matchbox_events: Option<Rc<RefCell<Box<MatchboxClient>>>>,
    ) -> Self {
        CoreGameState {
            board,
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
            (**self.matchbox_events.as_mut().unwrap())
                .borrow_mut()
                .try_recieve(&mut self.event_broker);

            if self.own_player_id.is_none() {
                self.own_player_id = (**self.matchbox_events.as_ref().unwrap())
                    .borrow()
                    .as_ref()
                    .get_own_player_index();
                info!("own player id: {:?}", self.own_player_id);
            }
        }

        for (i, text) in description(&self.render_context, (*self.board).borrow().as_ref())
            .iter()
            .enumerate()
        {
            draw_text(
                text.as_str(),
                10.,
                670. + (i * 50) as f32,
                50.,
                (*self.board).borrow().as_ref().current_team().color,
            );
        }

        match self.render_context.game_state {
            CoreGameSubstate::Wait => {
                if can_control_player(
                    (*self.board).borrow().as_ref(),
                    &mut self.own_player_id,
                    ONLINE,
                ) {
                    self.render_context.game_state = CoreGameSubstate::Place;
                }
            }
            CoreGameSubstate::Won(_) => {}
            _ => {
                handle_player_input(
                    &mut self.board,
                    &mut self.event_broker,
                    &mut self.render_context,
                    canvas,
                );
            }
        }

        check_if_somebody_won((*self.board).borrow().as_ref(), &mut self.render_context);

        if true {
            self.board_render = Rc::new(RefCell::new(Box::new(BoardRender::new(
                (*self.board).borrow().as_ref(),
            ))));
        }

        Option::None
    }

    fn render(&self, canvas: &Canvas2D) {
        (*self.board_render).borrow().as_ref().render(
            (*self.board).borrow().as_ref(),
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
        board: &mut Board,
        event_consumer: &mut EventBroker,
    ) -> CoreGameSubstate {
        if !board.has_cell(target_point) {
            return CoreGameSubstate::Place;
        }

        match self {
            CoreGameSubstate::Place => {
                if let Some(target_piece) = board.get_piece_at(target_point) {
                    if target_piece.team_id == board.current_team_index {
                        if target_piece.can_move() {
                            return CoreGameSubstate::Move(*target_point);
                        } else if target_piece.can_use_special() {
                            return CoreGameSubstate::Activate(*target_point);
                        }
                    }
                } else if board.unused_piece_available() {
                    let event = GameEvent::CompoundEvent(
                        vec![
                            GameEvent::RemoveUnusedPiece(board.current_team_index),
                            GameEvent::Place(
                                *target_point,
                                Piece::new(board.current_team_index, PieceKind::Simple),
                            ),
                        ],
                        CompoundEventType::Place,
                    );
                    event_consumer.handle_new_event(&event);
                }
            }
            CoreGameSubstate::Move(itself) => {
                if let Some(target_piece) = board.get_piece_at(target_point) {
                    if *itself == *target_point && target_piece.can_use_special() {
                        if let Some(activatable) = target_piece.activatable {
                            return match activatable.kind {
                                Power::Blast => {
                                    let mut game_events = vec![Exhaust(true, *target_point)];
                                    for point in activatable.range.reachable_points(
                                        target_point,
                                        board,
                                        &RangeContext::Special(*target_piece),
                                    ) {
                                        if let Some(piece) = board.get_piece_at(&point) {
                                            game_events.push(Remove(point, *piece));
                                        }
                                    }

                                    event_consumer.handle_new_event(&CompoundEvent(
                                        game_events,
                                        CompoundEventType::Attack,
                                    ));
                                    CoreGameSubstate::Place
                                }
                                Power::TargetedShoot => CoreGameSubstate::Activate(*target_point),
                            };
                        }
                    }
                    if target_piece.team_id == board.current_team_index && target_piece.can_move() {
                        return CoreGameSubstate::Move(*target_point);
                    }
                }

                let selected_piece = board.get_piece_at(&itself).unwrap();
                if let Some(m) = selected_piece.movement.as_ref() {
                    if m.range
                        .reachable_points(&itself, board, &RangeContext::Moving(*selected_piece))
                        .contains(target_point)
                    {
                        event_consumer.handle_new_event(&GameEvent::new_move(
                            *selected_piece,
                            *itself,
                            *target_point,
                        ));
                    }
                }
            }
            CoreGameSubstate::Activate(active_piece_pos) => {
                let active_piece = board.get_piece_at(active_piece_pos).unwrap();
                if let Some(target_piece) = board.get_piece_at(target_point) {
                    if target_piece.team_id != board.current_team_index
                        && active_piece.can_use_special()
                    {
                        event_consumer.handle_new_event(&CompoundEvent(
                            vec![
                                Exhaust(true, *active_piece_pos),
                                Remove(*target_point, *target_piece),
                            ],
                            CompoundEventType::Attack,
                        ));
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

    pub(crate) fn merge_patterns(board: &mut Board, event_consumer: &mut EventBroker) {
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
                                let matched_piece = board.get_piece_mut_at(point).unwrap();
                                event_consumer
                                    .handle_new_event(&GameEvent::Remove(*point, *matched_piece));
                                matched_piece.dying = true;
                            });

                            let new_piece = Piece::new(any_team_id, pattern.turn_into);

                            let new_piece_x = x as u8 + pattern.new_piece_relative_position.x;
                            let new_piece_y = y as u8 + pattern.new_piece_relative_position.y;

                            event_consumer.handle_new_event(&GameEvent::Place(
                                Point2::new(new_piece_x, new_piece_y),
                                new_piece,
                            ));

                            println!(
                                "Matched pattern at {}:{}; new piece at {}:{}",
                                x, y, new_piece_x, new_piece_y
                            );
                        }
                    }
                }
            }
        }
    }
}

fn check_if_somebody_won(board: &Board, render_context: &mut CustomRenderContext) {
    if board.placed_pieces(0).is_empty() || board.num_unused_pieces_of(1) >= 20 {
        info!("Team 1 won");
        render_context.game_state = CoreGameSubstate::Won(1);
    }
    if board.placed_pieces(1).is_empty() || board.num_unused_pieces_of(0) >= 20 {
        info!("Team 0 won");
        render_context.game_state = CoreGameSubstate::Won(0);
    }
}

fn description(render_context: &CustomRenderContext, board: &Board) -> Vec<String> {
    let mut description = vec![];

    match render_context.game_state {
        CoreGameSubstate::Place => {
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
                format!("The {} team won", board.teams[team].name)
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
    canvas: &Canvas2D,
) {
    if is_mouse_button_pressed(MouseButton::Left) {
        info!("mouse button pressed");
        let next_game_state = render_context.game_state.on_click(
            &cell_hovered(canvas),
            board.as_ref().borrow_mut().borrow_mut(),
            &mut event_broker,
        );
        info!("{:?} -> {:?}", render_context.game_state, next_game_state);
        render_context.game_state = next_game_state;
        render_context.reset_elapsed_time();

        event_broker.flush();
        CoreGameSubstate::merge_patterns((**board).borrow_mut().as_mut(), &mut event_broker);
        event_broker.commit(CompoundEventType::Merge);
        info!("finish mouse button action");
    }

    if is_key_pressed(KeyCode::Enter)
        || is_key_pressed(KeyCode::KpEnter)
        || render_context.button_next.clicked(canvas)
    {
        next_turn(&mut board, &mut event_broker);
        render_context.game_state = CoreGameSubstate::Wait;
    }

    if is_key_pressed(KeyCode::U) || render_context.button_undo.clicked(canvas) {
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
