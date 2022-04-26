use crate::{game_events::GameEvent::*, info, rand::rand, Board, BoardRender, Piece, Point2};

use nanoserde::{DeBin, SerBin};
use std::{cell::RefCell, mem, rc::Rc};
use crate::game::Game;

#[derive(Debug, Clone, SerBin, DeBin)]
pub enum GameEvent {
    Place(Point2, Piece),
    Remove(Point2, Piece),
    AddUnusedPiece(usize),
    RemoveUnusedPiece(usize),
    Exhaust(bool, Point2),
    UndoExhaustion(bool, Point2),
    CompoundEvent(Vec<GameEvent>, CompoundEventType),
    NextTurn,
}

impl GameEvent {
    pub fn new_move(piece: Piece, from: Point2, to: Point2) -> Self {
        CompoundEvent(
            vec![Remove(from, piece), Place(to, piece), Exhaust(false, to)],
            CompoundEventType::Move,
        )
    }
}

#[derive(Debug, Clone, SerBin, DeBin)]
pub struct GameEventObject {
    pub(crate) id: String,
    pub event: GameEvent,
}

impl GameEventObject {
    pub const OPCODE: i32 = 1;

    pub fn new(event: GameEvent) -> Self {
        GameEventObject {
            id: rand().to_string(),
            event,
        }
    }
}

#[derive(Debug, Clone, SerBin, DeBin)]
pub enum CompoundEventType {
    Merge,
    Attack,
    Place,
    Move,
    Undo(Box<CompoundEventType>),
    FinishTurn,
}

impl GameEvent {
    pub fn anti_event(&self) -> GameEvent {
        match self {
            //    Move(from, to) => Move(*to, *from),
            Place(at, piece) => Remove(*at, *piece),
            Remove(at, piece) => Place(*at, *piece),
            AddUnusedPiece(team_id) => RemoveUnusedPiece(*team_id),
            RemoveUnusedPiece(team_id) => AddUnusedPiece(*team_id),
            CompoundEvent(events, event_type) => CompoundEvent(
                events.iter().map(|e| e.anti_event()).rev().collect(),
                CompoundEventType::Undo(Box::new(event_type.clone())),
            ),
            Exhaust(special, point) => UndoExhaustion(*special, *point),
            UndoExhaustion(special, point) => Exhaust(*special, *point),
            NextTurn => {
                panic!("Cannot undo next turn");
            }
        }
    }
}

pub trait EventConsumer {
    fn handle_event(&mut self, event: &GameEventObject);
}

pub struct EventBroker {
    event_queue: Vec<GameEventObject>,
    past_events: Vec<GameEvent>,
    current_transaction: Vec<GameEvent>,
    pub(crate) subscribers: Vec<Box<dyn EventConsumer>>,
    pub(crate) committed_subscribers: Vec<Box<dyn EventConsumer>>,
}

impl EventBroker {
    pub(crate) fn new() -> Self {
        EventBroker {
            event_queue: vec![],
            past_events: vec![],
            current_transaction: vec![],
            subscribers: vec![],
            committed_subscribers: vec![],
        }
    }

    pub(crate) fn subscribe(&mut self, subscriber: Box<dyn EventConsumer>) {
        self.subscribers.push(subscriber);
    }

    pub(crate) fn subscribe_committed(&mut self, subscriber: Box<dyn EventConsumer>) {
        self.committed_subscribers.push(subscriber);
    }

    pub(crate) fn flush(&mut self) -> bool {
        if self.event_queue.is_empty() {
            return false;
        }

        for event in self.event_queue.drain(..) {
            self.subscribers
                .iter_mut()
                .for_each(|s| (*s).handle_event(&event));
            self.current_transaction.push(event.event);
        }

        true
    }

    pub fn undo(&mut self) {
        //let mut anti_events: Vec<GameEvent> = self.past_events.drain(..).map(|e| e.anti_event()).rev().collect();
        //for event in anti_events {
        self.assert_no_uncommitted_events();

        if let Some(event) = self.past_events.pop() {
            let event_object = GameEventObject::new(event.anti_event());
            self.handle_event(&event_object);
            self.commit(CompoundEventType::Undo(Box::from(
                CompoundEventType::FinishTurn,
            ))); // TODO: Use real type
        }
        //}
        //self.commit_without_history();
        //self.delete_history();
    }

    fn assert_no_uncommitted_events(&self) {
        if !self.current_transaction.is_empty() {
            panic!(
                "Unexpected uncommitted events: {:?}",
                self.current_transaction
            );
        }
    }

    pub fn commit(&mut self, c: CompoundEventType) {
        self.flush();
        let events: Vec<GameEvent> = self.current_transaction.drain(..).collect();

        if events.is_empty() {
            return;
        }

        info!("Committing...");

        let compound_event = CompoundEvent(events, c);
        self.past_events.push(compound_event.clone());
        self.committed_subscribers
            .iter_mut()
            .for_each(|s| (*s).handle_event(&GameEventObject::new(compound_event.clone())));

        info!("Compound event: {:?}", compound_event);
    }

    fn commit_without_history(&mut self) {
        self.current_transaction.clear();
    }

    pub fn delete_history(&mut self) {
        self.assert_no_uncommitted_events();
        self.past_events.clear();
    }

    pub fn handle_new_event(&mut self, event: &GameEvent) {
        let event_object = GameEventObject::new(event.clone());
        self.handle_event(&event_object);
    }

    pub fn handle_remote_event(&mut self, event: &GameEventObject) {
        self.handle_event(&event);
        self.flush();
        self.commit_without_history();
    }
}

impl EventConsumer for EventBroker {
    fn handle_event(&mut self, event: &GameEventObject) {
        self.event_queue.push(event.clone());
    }
}

pub struct BoardEventConsumer {
    pub(crate) game: Rc<RefCell<Box<Game>>>,
}

pub struct RenderEventConsumer {
    pub(crate) board_render: Rc<RefCell<Box<BoardRender>>>,
}

impl RenderEventConsumer {
    pub(crate) fn handle_event_internal(&self, events: &Vec<GameEvent>, t: &CompoundEventType) {
        let mut board_render = (*self.board_render).borrow_mut();

        match t {
            CompoundEventType::Merge => {}
            CompoundEventType::Attack => {}
            CompoundEventType::Place => {
                for event in events {
                    if let Place(point, piece) = event {
                        let mut unused = board_render.unused_pieces[piece.team_id]
                            .pop()
                            .expect("No unused piece left in BoardRender");
                        unused.move_towards(point);
                        //let color = board_render.team_colors[piece.team_id];
                        //board_render.placed_pieces.push(PieceRender::from_piece(point, piece, color))
                        board_render.placed_pieces.insert(*point, unused);
                    }
                }
            }
            CompoundEventType::Move => {}
            CompoundEventType::Undo(_) => {}
            CompoundEventType::FinishTurn => {}
        }
    }
}

impl BoardEventConsumer {
    fn handle_event_internal(&mut self, event: &GameEvent) {
        let mut game = (*self.game).borrow_mut();
        let board = &mut game.board;

        match event {
            GameEvent::Place(at, piece) => {
                board.place_piece_at(*piece, at);
            }
            GameEvent::Remove(at, _) => {
                board.remove_piece_at(at);
            }
            GameEvent::AddUnusedPiece(team_id) => {
                game.add_unused_piece_for(*team_id);
            }
            GameEvent::RemoveUnusedPiece(team_id) => {
                game.remove_unused_piece(*team_id);
            }
            Exhaust(special, point) => {
                let exhaustion = &mut board
                    .get_piece_mut_at(point)
                    .expect(&*format!(
                        "Can't execute {:?} for non-existing piece at {:?}",
                        event, point
                    ))
                    .exhaustion;

                if *special {
                    exhaustion.on_attack();
                } else {
                    println!("Before exhaustion {}", exhaustion.can_move());
                    exhaustion.on_move();
                    println!("After exhaustion {}", exhaustion.can_move());
                }
            }
            UndoExhaustion(special, point) => {
                let exhaustion = &mut board
                    .get_piece_mut_at(point)
                    .expect(&*format!(
                        "Can't execute {:?} for non-existing piece at {:?}",
                        event, point
                    ))
                    .exhaustion;

                if *special {
                    exhaustion.undo_attack();
                } else {
                    exhaustion.undo_move();
                }
            }
            GameEvent::CompoundEvent(_events, _) => {}
            NextTurn => {
                game.next_team();
            }
        }

        mem::drop(game);

        match event {
            GameEvent::CompoundEvent(events, _) => {
                events.iter().for_each(|e| self.handle_event_internal(e))
            }
            _ => {}
        }
    }
}

impl EventConsumer for BoardEventConsumer {
    fn handle_event(&mut self, event_object: &GameEventObject) {
        let event = &event_object.event;
        println!("Handling event {:?}", event);
        self.handle_event_internal(event);
    }
}

impl EventConsumer for RenderEventConsumer {
    fn handle_event(&mut self, event: &GameEventObject) {
        match &event.event {
            GameEvent::CompoundEvent(events, t) => {
                self.handle_event_internal(events, t);
            }
            _ => {}
        }
    }
}
