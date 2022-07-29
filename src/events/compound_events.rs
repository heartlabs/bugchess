use crate::{AtomicEvent, Exhaustion, Piece, PieceKind, Point2};
use nanoserde::{DeBin, SerBin};
use std::fmt::Debug;

#[derive(Debug, Clone, SerBin, DeBin)]
pub struct MergeCompoundEvent {
    events: Vec<AtomicEvent>,
    was_flushed: bool,
}

#[derive(Debug, Clone, SerBin, DeBin)]
pub struct AttackCompoundEvent {
    events: Vec<AtomicEvent>,
    pub piece_kind: PieceKind,
    merge_events: MergeCompoundEvent,
    was_flushed: bool,
}

#[derive(Debug, Clone, SerBin, DeBin)]
pub struct PlaceCompoundEvent {
    events: Vec<AtomicEvent>,
    merge_events: MergeCompoundEvent,
    was_flushed: bool,
}

#[derive(Debug, Clone, SerBin, DeBin)]
pub struct MoveCompoundEvent {
    events: Vec<AtomicEvent>,
    merge_events: MergeCompoundEvent,
    was_flushed: bool,
}

#[derive(Debug, Clone, SerBin, DeBin)]
pub struct UndoCompoundEvent {
    events: Vec<AtomicEvent>,
    undone: Box<GameAction>,
    was_flushed: bool,
}

#[derive(Debug, Clone, SerBin, DeBin)]
pub struct FinishTurnCompoundEvent {
    events: Vec<AtomicEvent>,
    was_flushed: bool,
}

impl MergeCompoundEvent {
    fn new() -> Self {
        MergeCompoundEvent {
            events: vec![],
            was_flushed: false,
        }
    }
}

impl CompoundEvent for MergeCompoundEvent {
    fn get_events(&self) -> Vec<AtomicEvent> {
        self.events.clone()
    }

    fn push_event(&mut self, event: AtomicEvent) {
        assert!(
            !self.was_flushed,
            "Cannot push event after already being flushed"
        );
        self.events.push(event);
    }

    fn flush(&mut self) -> Vec<AtomicEvent> {
        if self.was_flushed {
            return vec![];
        }

        self.was_flushed = true;

        self.get_events()
    }
}

impl CompoundEvent for AttackCompoundEvent {
    fn get_events(&self) -> Vec<AtomicEvent> {
        let mut all_events: Vec<AtomicEvent> = vec![];
        all_events.extend(&self.events);
        all_events.extend(&self.merge_events.events);
        all_events
    }

    fn push_event(&mut self, event: AtomicEvent) {
        if self.was_flushed {
            self.merge_events.push_event(event);
        } else {
            self.events.push(event);
        }
    }

    fn flush(&mut self) -> Vec<AtomicEvent> {
        if self.was_flushed {
            return self.merge_events.flush();
        }

        self.get_events()
    }
}

impl CompoundEvent for PlaceCompoundEvent {
    fn get_events(&self) -> Vec<AtomicEvent> {
        let mut all_events: Vec<AtomicEvent> = vec![];
        all_events.extend(&self.events);
        all_events.extend(&self.merge_events.events);
        all_events
    }

    fn push_event(&mut self, event: AtomicEvent) {
        if self.was_flushed {
            self.merge_events.push_event(event);
        } else {
            self.events.push(event);
        }
    }

    fn flush(&mut self) -> Vec<AtomicEvent> {
        if self.was_flushed {
            return self.merge_events.flush();
        }

        self.was_flushed = true;

        self.get_events()
    }
}

impl CompoundEvent for MoveCompoundEvent {
    fn get_events(&self) -> Vec<AtomicEvent> {
        let mut all_events: Vec<AtomicEvent> = vec![];
        all_events.extend(&self.events);
        all_events.extend(&self.merge_events.events);
        all_events
    }

    fn push_event(&mut self, event: AtomicEvent) {
        if self.was_flushed {
            self.merge_events.push_event(event);
        } else {
            self.events.push(event);
        }
    }

    fn flush(&mut self) -> Vec<AtomicEvent> {
        if self.was_flushed {
            return self.merge_events.flush();
        }

        self.was_flushed = true;

        self.get_events()
    }
}

impl CompoundEvent for UndoCompoundEvent {
    fn get_events(&self) -> Vec<AtomicEvent> {
        self.events.clone()
    }

    fn push_event(&mut self, event: AtomicEvent) {
        assert!(
            !self.was_flushed,
            "Cannot push event after already being flushed"
        );
        self.events.push(event);
    }

    fn flush(&mut self) -> Vec<AtomicEvent> {
        if self.was_flushed {
            return vec![];
        }

        self.was_flushed = true;

        self.get_events()
    }
}

impl CompoundEvent for FinishTurnCompoundEvent {
    fn get_events(&self) -> Vec<AtomicEvent> {
        self.events.clone()
    }

    fn push_event(&mut self, event: AtomicEvent) {
        assert!(
            !self.was_flushed,
            "Cannot push event after already being flushed"
        );
        self.events.push(event);
    }

    fn flush(&mut self) -> Vec<AtomicEvent> {
        if self.was_flushed {
            return vec![];
        }

        self.was_flushed = true;

        self.get_events()
    }
}

pub struct FinishTurnBuilder {
    event: FinishTurnCompoundEvent
}

impl FinishTurnBuilder {
    pub fn build(self) -> GameAction {
        GameAction::FinishTurn(self.event)
    }

    pub fn place_piece(&mut self, point: Point2, piece: Piece) -> &mut Self {
        self.event.events.push(AtomicEvent::Place(point, piece));

        self
    }

    pub fn add_unused_piece(&mut self, team: usize) -> &mut Self {
        self.event.events.push(AtomicEvent::AddUnusedPiece(team));

        self
    }

    pub fn change_exhaustion(&mut self, from: Exhaustion, to: Exhaustion, at: Point2) -> &mut Self {
        self.event.events.push(AtomicEvent::ChangeExhaustion(from,to,at));

        self
    }
}

pub trait CompoundEvent: Debug {
    fn get_events(&self) -> Vec<AtomicEvent>;

    fn push_event(&mut self, event: AtomicEvent); // TODO remove
                                                  //    fn get_event_type(&self) -> &CompoundEventType;

    fn flush(&mut self) -> Vec<AtomicEvent>;
}

#[derive(Debug, Clone, SerBin, DeBin)]
pub enum GameAction {
    Attack(AttackCompoundEvent),
    Place(PlaceCompoundEvent),
    Move(MoveCompoundEvent),
    Undo(UndoCompoundEvent),
    FinishTurn(FinishTurnCompoundEvent),
}

impl GameAction {
    pub fn attack(piece_kind: PieceKind) -> GameAction {
        GameAction::Attack(AttackCompoundEvent {
            events: vec![],
            piece_kind,
            merge_events: MergeCompoundEvent::new(),
            was_flushed: false,
        })
    }

    pub fn place() -> GameAction {
        GameAction::Place(PlaceCompoundEvent {
            events: vec![],
            merge_events: MergeCompoundEvent::new(),
            was_flushed: false,
        })
    }

    pub fn moving() -> GameAction {
        GameAction::Move(MoveCompoundEvent {
            events: vec![],
            merge_events: MergeCompoundEvent::new(),
            was_flushed: false,
        })
    }

    pub fn undo(undone: Box<GameAction>) -> GameAction {
        GameAction::Undo(UndoCompoundEvent {
            events: vec![],
            undone,
            was_flushed: false,
        })
    }

    pub fn finish_turn() -> FinishTurnBuilder {
        FinishTurnBuilder {
            event: FinishTurnCompoundEvent {
                    events: vec![AtomicEvent::NextTurn],
                    was_flushed: false,
            }
        }
    }

    pub fn get_compound_event(&self) -> Box<&dyn CompoundEvent> {
        match self {
            GameAction::Attack(e) => Box::new(e),
            GameAction::Place(e) => Box::new(e),
            GameAction::Move(e) => Box::new(e),
            GameAction::Undo(e) => Box::new(e),
            GameAction::FinishTurn(e) => Box::new(e),
        }
    }

    pub fn get_compound_event_mut(&mut self) -> Box<&mut dyn CompoundEvent> {
        match self {
            GameAction::Attack(e) => Box::new(e),
            GameAction::Place(e) => Box::new(e),
            GameAction::Move(e) => Box::new(e),
            GameAction::Undo(e) => Box::new(e),
            GameAction::FinishTurn(e) => Box::new(e),
        }
    }

    pub fn anti_event(&self) -> GameAction {
        GameAction::Undo(UndoCompoundEvent {
            events: self
                .get_compound_event()
                .get_events()
                .iter()
                .map(|e| e.anti_event())
                .rev()
                .collect(),
            undone: Box::new(self.clone()),
            was_flushed: false,
        })
    }
}
