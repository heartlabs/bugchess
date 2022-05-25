use crate::{
    game_events::GameEvent::*,
    game_logic::{board::*, game::*, piece::*},
    info,
    rand::rand,
};

use crate::{rendering::BoardRender, CompoundEventType::Undo};
use macroquad::{logging::warn, ui::Drag::No};
use nanoserde::{DeBin, SerBin};
use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    mem,
    rc::Rc,
    vec::Drain,
};

#[derive(Debug, Clone, SerBin, DeBin)]
pub struct CompoundEvent {
    pub events: Vec<GameEvent>,
    pub kind: CompoundEventType,
}

#[derive(Debug, Copy, Clone, SerBin, DeBin)]
pub enum GameEvent {
    Place(Point2, Piece),
    Remove(Point2, Piece),
    AddUnusedPiece(usize),
    RemoveUnusedPiece(usize),
    ChangeExhaustion(Exhaustion, Exhaustion, Point2), // From, To, At
    AddEffect(EffectKind, Point2),
    RemoveEffect(EffectKind, Point2),
    NextTurn,
}

#[derive(Debug, Clone, SerBin, DeBin)]
pub struct GameEventObject {
    pub id: String,
    pub sender: String,
    pub event: CompoundEvent,
}

impl GameEventObject {
    pub const OPCODE: i32 = 1;

    pub fn new(event: CompoundEvent, sender: &String) -> Self {
        GameEventObject {
            id: rand().to_string(),
            sender: sender.clone(),
            event,
        }
    }
}

#[derive(Debug, Clone, SerBin, DeBin)]
pub enum CompoundEventType {
    Attack(PieceKind),
    Place,
    Move,
    Undo(Box<CompoundEventType>),
    FinishTurn,
}

impl CompoundEvent {
    pub fn anti_event(&self) -> CompoundEvent {
        CompoundEvent {
            events: self.events.iter().map(|e| e.anti_event()).rev().collect(),
            kind: CompoundEventType::Undo(Box::new(self.kind.clone())),
        }
    }
}

impl GameEvent {
    pub fn anti_event(&self) -> GameEvent {
        match self {
            //    Move(from, to) => Move(*to, *from),
            Place(at, piece) => Remove(*at, *piece),
            Remove(at, piece) => Place(*at, *piece),
            AddUnusedPiece(team_id) => RemoveUnusedPiece(*team_id),
            RemoveUnusedPiece(team_id) => AddUnusedPiece(*team_id),
            ChangeExhaustion(from, to, point) => ChangeExhaustion(*to, *from, *point),
            AddEffect(kind, at) => RemoveEffect(*kind, *at),
            RemoveEffect(kind, at) => AddEffect(*kind, *at),
            NextTurn => {
                panic!("Cannot undo next turn");
            }
        }
    }
}

pub trait EventConsumer {
    fn handle_event(&mut self, event: &GameEventObject);
}

pub struct EventComposer {
    unflushed: Vec<GameEvent>,
    current_transaction: Vec<GameEvent>,
    current_transaction_type: Option<CompoundEventType>,
    committed: Vec<CompoundEvent>,
    pub(crate) game: Rc<RefCell<Box<Game>>>,
}

impl EventComposer {
    pub fn new(game: Rc<RefCell<Box<Game>>>) -> Self {
        Self {
            unflushed: vec![],
            current_transaction: vec![],
            current_transaction_type: None,
            committed: vec![],
            game,
        }
    }

    pub fn drain_commits(&mut self) -> Vec<CompoundEvent> {
        assert!(
            self.unflushed.is_empty(),
            "There are still unflushed events"
        );

        self.committed.drain(..).collect()
    }

    pub fn init_new_transaction(&mut self, mut events: Vec<GameEvent>, c: CompoundEventType) {
        self.start_transaction(c);
        for event in events {
            self.push_event(event);
        }
    }

    pub(crate) fn push_event(&mut self, event: GameEvent) {
        self.unflushed.push(event);
        self.current_transaction.push(event);
    }

    pub(crate) fn flush(&mut self) -> bool {
        if self.unflushed.is_empty() {
            return false;
        }

        for event in self.unflushed.drain(..) {
            BoardEventConsumer::handle_event_internal((*self.game).borrow_mut().as_mut(), &event);
        }

        true
    }

    fn assert_no_uncommitted_events(&self) {
        if !self.current_transaction.is_empty() {
            panic!(
                "Unexpected uncommitted events: {:?}",
                self.current_transaction
            );
        }
    }

    pub fn start_transaction(&mut self, c: CompoundEventType) {
        self.assert_no_uncommitted_events();

        self.current_transaction_type = Some(c);
    }

    pub fn commit(&mut self) {
        if self.current_transaction.is_empty() {
            return;
        }

        let events: Vec<GameEvent> = self.current_transaction.drain(..).collect();

        let event_type = self
            .current_transaction_type
            .take()
            .expect("Can't commit because there was no transaction started");
        let compound_event = CompoundEvent {
            events,
            kind: event_type,
        };

        info!("Compound event: {:?}", compound_event);

        self.committed.push(compound_event);
    }

    fn commit_without_history(&mut self) {
        self.current_transaction.clear();
    }
}

pub struct EventBroker {
    sender_id: String,
    past_events: Vec<CompoundEvent>,
    pub(crate) subscribers: Vec<Box<dyn EventConsumer>>,
}

impl EventBroker {
    pub(crate) fn new(sender_id: String) -> Self {
        EventBroker {
            sender_id,
            past_events: vec![],
            subscribers: vec![],
        }
    }

    pub(crate) fn subscribe_committed(&mut self, subscriber: Box<dyn EventConsumer>) {
        self.subscribers.push(subscriber);
    }

    pub fn undo(&mut self) {
        if let Some(event) = self.past_events.pop() {
            let event_object = GameEventObject::new(event.anti_event(), &self.sender_id);
            self.handle_event(&event_object);
        }
    }

    pub fn delete_history(&mut self) {
        self.past_events.clear();
    }

    pub fn handle_new_event(&mut self, event: &CompoundEvent) {
        self.past_events.push(event.clone());

        let event_object = GameEventObject::new(event.clone(), &self.sender_id);
        self.handle_event(&event_object);
    }

    pub fn handle_remote_event(&mut self, event: &GameEventObject) {
        self.handle_event(&event);
    }
}

impl EventConsumer for EventBroker {
    fn handle_event(&mut self, event: &GameEventObject) {
        self.subscribers
            .iter_mut()
            .for_each(|s| (*s).handle_event(event));

        if let CompoundEventType::FinishTurn = event.event.kind {
            self.delete_history();
        }
    }
}

pub struct BoardEventConsumer {
    pub own_sender_id: String,
    pub(crate) game: Rc<RefCell<Box<Game>>>,
}

impl EventConsumer for BoardEventConsumer {
    fn handle_event(&mut self, event_object: &GameEventObject) {
        if !matches!(event_object.event.kind, Undo(_)) && event_object.sender == self.own_sender_id {
            return;
        }

        let event = &event_object.event;
        println!("Handling event {:?}", event);

        event.events.iter().for_each(|e| {
            BoardEventConsumer::handle_event_internal((*self.game).borrow_mut().as_mut(), e)
        });
    }
}
impl BoardEventConsumer {
    fn handle_event_internal(game: &mut Game, event: &GameEvent) {
        let board = &mut game.board;

        match event {
            Place(at, piece) => {
                board.place_piece_at(*piece, at);
            }
            Remove(at, _) => {
                board.remove_piece_at(at);
            }
            AddUnusedPiece(team_id) => {
                game.add_unused_piece_for(*team_id);
            }
            RemoveUnusedPiece(team_id) => {
                game.remove_unused_piece(*team_id);
            }
            ChangeExhaustion(from, to, point) => {
                let piece = &mut board
                    .get_piece_mut_at(point)
                    .expect(&*format!(
                        "Can't execute {:?} for non-existing piece at {:?}",
                        event, point
                    ));

                assert_eq!(from, &piece.exhaustion, "Expected piece at {:?} to have exhaustion state {:?} but it had {:?}", point, from, piece.exhaustion);

                piece.exhaustion = *to;
            }
            AddEffect(kind, at) => {board.add_effect(*kind, at);}
            RemoveEffect(kind, at) => {board.remove_effect(kind, at)}
            NextTurn => {
                warn!("NEXT TURN");
                game.next_team();
            }
        }
    }
}
