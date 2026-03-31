use crate::actions::compound_events::GameAction;

pub struct UndoManager {
    past_events: Vec<GameAction>,
    turn_boundary: usize,
}

impl UndoManager {
    pub fn new() -> Self {
        UndoManager {
            past_events: vec![],
            turn_boundary: 0,
        }
    }

    pub fn push(&mut self, event: GameAction) {
        self.past_events.push(event);
    }

    pub fn mark_turn_boundary(&mut self) {
        self.turn_boundary = self.past_events.len();
    }

    pub fn undo(&mut self) -> Option<GameAction> {
        if self.past_events.len() <= self.turn_boundary {
            return None;
        }
        self.past_events.pop().map(|e| e.anti_event())
    }
}
