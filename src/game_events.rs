use crate::{
    game_events::GameEvent::*,
    game_logic::{board::*, game::*, piece::*},
    info,
    rand::rand,
};

use macroquad::{logging::warn};
use nanoserde::{DeBin, SerBin};
use std::{
    cell::RefCell,
    rc::Rc,
};
use std::fmt::Debug;
use std::slice::Iter;


#[derive(Debug, Clone, SerBin, DeBin)]
pub struct MergeCompoundEvent {
    events: Vec<GameEvent>,
    was_flushed: bool,
}
#[derive(Debug, Clone, SerBin, DeBin)]
pub struct AttackCompoundEvent {
    events: Vec<GameEvent>,
    pub piece_kind: PieceKind,
    merge_events: MergeCompoundEvent,
    was_flushed: bool,
}
#[derive(Debug, Clone, SerBin, DeBin)]
pub struct PlaceCompoundEvent {
    events: Vec<GameEvent>,
    merge_events: MergeCompoundEvent,
    was_flushed: bool,
}
#[derive(Debug, Clone, SerBin, DeBin)]
pub struct MoveCompoundEvent {
    events: Vec<GameEvent>,
    merge_events: MergeCompoundEvent,
    was_flushed: bool,
}
#[derive(Debug, Clone, SerBin, DeBin)]
pub struct UndoCompoundEvent {
    events: Vec<GameEvent>,
    undone: Box<CompoundEventType>,
    was_flushed: bool,
}
#[derive(Debug, Clone, SerBin, DeBin)]
pub struct FinishTurnCompoundEvent {
    events: Vec<GameEvent>,
    was_flushed: bool,
}

impl MergeCompoundEvent {
    fn new() -> Self {
        MergeCompoundEvent { events: vec![], was_flushed: false }
    }
}

impl CompoundEvent for MergeCompoundEvent {
    fn get_events(&self) -> Vec<GameEvent> {
        self.events.clone()
    }

    fn push_event(&mut self, event: GameEvent) {
        assert!(!self.was_flushed, "Cannot push event after already being flushed");
        self.events.push(event);
    }

    fn flush(&mut self, game: &mut Game) -> bool {
        if self.was_flushed {
            return false;
        }

        self.was_flushed = true;

        for event in self.get_events() {
            BoardEventConsumer::handle_event_internal(game, &event);
        }

        true
    }
}

impl CompoundEvent for AttackCompoundEvent {
    fn get_events(&self) -> Vec<GameEvent> {
        let mut all_events: Vec<GameEvent> = vec![];
        all_events.extend(&self.events);
        all_events.extend(&self.merge_events.events);
        all_events
    }

    fn push_event(&mut self, event: GameEvent) {
        if self.was_flushed {
            self.merge_events.push_event(event);
        } else {
            self.events.push(event);
        }
    }

    fn flush(&mut self, game: &mut Game) -> bool {
        if self.was_flushed {
            return self.merge_events.flush(game);
        }

        for event in self.get_events() {
            BoardEventConsumer::handle_event_internal(game, &event);
        }

        true
    }
}

impl CompoundEvent for PlaceCompoundEvent {
    fn get_events(&self) -> Vec<GameEvent> {
        let mut all_events: Vec<GameEvent> = vec![];
        all_events.extend(&self.events);
        all_events.extend(&self.merge_events.events);
        all_events
    }

    fn push_event(&mut self, event: GameEvent) {
        if self.was_flushed {
            self.merge_events.push_event(event);
        } else {
            self.events.push(event);
        }
    }

    fn flush(&mut self, game: &mut Game) -> bool {
        if self.was_flushed {
            return self.merge_events.flush(game);
        }

        self.was_flushed = true;

        for event in self.get_events() {
            BoardEventConsumer::handle_event_internal(game, &event);
        }

        true
    }
}
impl CompoundEvent for MoveCompoundEvent {
    fn get_events(&self) -> Vec<GameEvent> {
        let mut all_events: Vec<GameEvent> = vec![];
        all_events.extend(&self.events);
        all_events.extend(&self.merge_events.events);
        all_events
    }

    fn push_event(&mut self, event: GameEvent) {
        if self.was_flushed {
            self.merge_events.push_event(event);
        } else {
            self.events.push(event);
        }
    }

    fn flush(&mut self, game: &mut Game) -> bool {
        if self.was_flushed {
            return self.merge_events.flush(game);
        }

        self.was_flushed = true;

        for event in self.get_events() {
            BoardEventConsumer::handle_event_internal(game, &event);
        }

        true
    }
}
impl CompoundEvent for UndoCompoundEvent {
    fn get_events(&self) -> Vec<GameEvent> {
        self.events.clone()
    }

    fn push_event(&mut self, event: GameEvent) {
        assert!(!self.was_flushed, "Cannot push event after already being flushed");
        self.events.push(event);
    }

    fn flush(&mut self, game: &mut Game) -> bool {
        if self.was_flushed {
            return false;
        }

        self.was_flushed = true;

        for event in self.get_events() {
            BoardEventConsumer::handle_event_internal(game, &event);
        }

        true
    }
}
impl CompoundEvent for FinishTurnCompoundEvent {
    fn get_events(&self) -> Vec<GameEvent> {
        self.events.clone()
    }

    fn push_event(&mut self, event: GameEvent) {
        assert!(!self.was_flushed, "Cannot push event after already being flushed");
        self.events.push(event);
    }

    fn flush(&mut self, game: &mut Game) -> bool {
        if self.was_flushed {
            return false;
        }

        self.was_flushed = true;

        for event in self.get_events() {
            BoardEventConsumer::handle_event_internal(game, &event);
        }

        true
    }
}

pub trait CompoundEvent : Debug {
    fn get_events(&self) -> Vec<GameEvent>;

    fn push_event(&mut self, event: GameEvent); // TODO remove
//    fn get_event_type(&self) -> &CompoundEventType;

    fn flush(&mut self, game: &mut Game) -> bool;
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
    pub event: CompoundEventType,
}

impl GameEventObject {
    pub const OPCODE: i32 = 1;

    pub fn new(event: CompoundEventType, sender: &String) -> Self {
        GameEventObject {
            id: rand().to_string(),
            sender: sender.clone(),
            event,
        }
    }
}

#[derive(Debug, Clone, SerBin, DeBin)]
pub enum CompoundEventType { // TODO Rename to GameAction
    Attack(AttackCompoundEvent),
    Place(PlaceCompoundEvent),
    Move(MoveCompoundEvent),
    Undo(UndoCompoundEvent),
    FinishTurn(FinishTurnCompoundEvent),
}

impl CompoundEventType {


    pub fn attack(piece_kind: PieceKind) -> CompoundEventType {
        CompoundEventType::Attack(AttackCompoundEvent {events: vec![], piece_kind, merge_events: MergeCompoundEvent::new(), was_flushed: false })
    }

    pub fn place() -> CompoundEventType {
        CompoundEventType::Place(PlaceCompoundEvent {events: vec![], merge_events: MergeCompoundEvent::new(), was_flushed: false})
    }

    pub fn moving() -> CompoundEventType {
        CompoundEventType::Move(MoveCompoundEvent {events: vec![], merge_events: MergeCompoundEvent::new(), was_flushed: false})
    }

    pub fn undo(undone: Box<CompoundEventType>) -> CompoundEventType {
        CompoundEventType::Undo(UndoCompoundEvent {events: vec![], undone, was_flushed: false })
    }

    pub fn finish_turn() -> CompoundEventType {
        CompoundEventType::FinishTurn(FinishTurnCompoundEvent {events: vec![], was_flushed: false })
    }

    pub fn get_compound_event(&self) -> Box<&dyn CompoundEvent> {
        match self {
            CompoundEventType::Attack(e) => Box::new(e),
            CompoundEventType::Place(e) => Box::new(e),
            CompoundEventType::Move(e) => Box::new(e),
            CompoundEventType::Undo(e) => Box::new(e),
            CompoundEventType::FinishTurn(e) => Box::new(e),
        }
    }

    pub fn get_compound_event_mut(&mut self) -> Box<&mut dyn CompoundEvent> {
        match self {
            CompoundEventType::Attack(e) => Box::new(e),
            CompoundEventType::Place(e) => Box::new(e),
            CompoundEventType::Move(e) => Box::new(e),
            CompoundEventType::Undo(e) => Box::new(e),
            CompoundEventType::FinishTurn(e) => Box::new(e),
        }
    }

    pub fn anti_event(&self) -> CompoundEventType {
        CompoundEventType::Undo(UndoCompoundEvent {
            events: self.get_compound_event().get_events().iter().map(|e| e.anti_event()).rev().collect(),
            undone: Box::new(self.clone()),
            was_flushed: false
        })
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

pub struct EventBroker {
    sender_id: String,
    past_events: Vec<CompoundEventType>,
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

    pub fn handle_new_event(&mut self, event: &CompoundEventType) {
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

        if let CompoundEventType::FinishTurn(_) = event.event {
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
        if !matches!(event_object.event, CompoundEventType::Undo(_)) && event_object.sender == self.own_sender_id {
            return;
        }

        let event = &event_object.event;
        println!("Handling event {:?}", event);

        event.get_compound_event().get_events().iter().for_each(|e| {
            BoardEventConsumer::handle_event_internal((*self.game).borrow_mut().as_mut(), e)
        });
    }
}
impl BoardEventConsumer {
    fn handle_event_internal(game: &mut Game, event: &GameEvent) {
        println!("Handling event INTERNAL {:?}", event);
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
