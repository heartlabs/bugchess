use crate::{
    events::actions::{compound_events::CompoundEvent, merge::MergeCompoundEvent},
    AtomicEvent, GameAction,
};
use game_logic::piece::PieceKind;
use nanoserde::{DeBin, SerBin};

#[derive(Debug, Clone, SerBin, DeBin)]
pub struct AttackCompoundEvent {
    events: Vec<AtomicEvent>,
    pub piece_kind: PieceKind,
    merge_events: MergeCompoundEvent,
    was_flushed: bool,
}

pub struct AttackBuilder {
    event: AttackCompoundEvent,
}

impl AttackBuilder {
    pub fn build(self) -> GameAction {
        GameAction::Attack(self.event)
    }
}

impl AttackBuilder {
    pub(crate) fn new(piece_kind: PieceKind) -> Self {
        AttackBuilder {
            event: AttackCompoundEvent {
                events: vec![],
                piece_kind,
                merge_events: MergeCompoundEvent::new(),
                was_flushed: false,
            },
        }
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
