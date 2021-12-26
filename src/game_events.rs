use std::cell::RefCell;
use std::mem;
use std::rc::Rc;
use crate::{Board, Piece, Point2};
use crate::game_events::GameEvent::*;

#[derive(Debug,Clone)]
pub enum GameEvent {
    Move(Point2, Point2),
    Place(Point2, Piece),
    Remove(Point2, Piece),
    AddUnusedPiece(usize),
    RemoveUnusedPiece(usize),
    CompoundEvent(Vec<GameEvent>, CompoundEventType)
}

#[derive(Debug,Clone)]
pub enum CompoundEventType {
    Merge,
    Attack,
    Place,
    Undo
}

impl GameEvent {
    pub fn anti_event(&self) -> GameEvent {
        match self {
            Move(from, to) => Move(*to, *from),
            Place(at, piece) => Remove(*at, *piece),
            Remove(at, piece) => Place(*at, *piece),
            AddUnusedPiece(team_id) => RemoveUnusedPiece(*team_id),
            RemoveUnusedPiece(team_id) => AddUnusedPiece(*team_id),
            CompoundEvent(events, _) => CompoundEvent(events.iter().map(|e| e.anti_event()).rev().collect(), CompoundEventType::Undo)
        }
    }
}

pub trait EventConsumer {
    fn handle_event(&mut self, event: &GameEvent);
}

pub struct EventBroker {
    event_queue: Vec<GameEvent>,
    past_events: Vec<GameEvent>,
    current_transaction: Vec<GameEvent>,
    pub(crate) subscribers: Vec<Box<dyn EventConsumer>>
    //pub(crate) subscribers: Vec<Rc<RefCell<Box<Board>>>>
}

impl EventBroker {
    pub(crate) fn new() -> Self {
        EventBroker {
            event_queue: vec![],
            past_events: vec![],
            current_transaction: vec![],
            subscribers: vec![],
        }
    }

    pub(crate) fn subscribe(&mut self, subscriber: Box<dyn EventConsumer>){
        self.subscribers.push(subscriber);
    }

    pub(crate) fn flush(&mut self) -> bool {
        if self.event_queue.is_empty() {
            return false
        }

        for event in self.event_queue.drain(..) {
            self.subscribers.iter_mut().for_each(|s| (*s).handle_event(&event));
            self.current_transaction.push(event);
        }

        true
    }

    pub fn undo(&mut self) {
        //let mut anti_events: Vec<GameEvent> = self.past_events.drain(..).map(|e| e.anti_event()).rev().collect();
        //for event in anti_events {
        self.assert_no_uncommitted_events();

        if let Some(event) = self.past_events.pop() {
            self.handle_event(&event.anti_event());
        }
        //}
        self.flush();
        self.commit_without_history();
        //self.delete_history();
    }

    fn assert_no_uncommitted_events(&self) {
        if !self.current_transaction.is_empty() {
            panic!("Unexpected uncommitted events: {:?}", self.current_transaction);
        }
    }

    pub fn commit(&mut self, c: Option<CompoundEventType>) {
        self.flush();
        let mut compound_event: Vec<GameEvent> = self.current_transaction.drain(..).collect();
        if let Some(cet) = c {
            self.past_events.push(CompoundEvent(compound_event, cet));
        } else {
            self.past_events.append(&mut compound_event);
        }
    }

    fn commit_without_history(&mut self) {
        self.current_transaction.clear();
    }

    pub fn delete_history(&mut self) {
        self.assert_no_uncommitted_events();
        self.past_events.clear();
    }
}

impl EventConsumer for EventBroker {
    fn handle_event(&mut self, event: &GameEvent) {
        self.event_queue.push(event.clone());
    }
}

pub struct BoardEventConsumer {
    pub(crate) board: Rc<RefCell<Box<Board>>>
}

impl EventConsumer for BoardEventConsumer {
    fn handle_event(&mut self, event: &GameEvent) {
        println!("Handling event {:?}", event);
        let mut board = (*self.board).borrow_mut();

        match event {
            GameEvent::Move(from, to) => {board.move_piece(from.x, from.y, to.x, to.y);}
            GameEvent::Place(at, piece) => {board.place_piece(*piece, at.x, at.y);}
            GameEvent::Remove(at, _) => {board.remove_piece(at.x, at.y);}
            GameEvent::AddUnusedPiece(team_id) => {board.add_unused_piece_for(*team_id);}
            GameEvent::RemoveUnusedPiece(team_id) => {board.remove_unused_piece(*team_id);}
            GameEvent::CompoundEvent(events, _) => {}
        }

        mem::drop(board);

        match event {
            GameEvent::CompoundEvent(events, _) => {events.iter().for_each(|e| self.handle_event(e))}
            _ => {}
        }
    }
}
