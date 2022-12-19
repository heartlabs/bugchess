use crate::{
    actions::compound_events::GameAction,
    game_events::{EventConsumer, GameEventObject, PlayerAction, Event},
};

pub struct GameHistory {
    start_of_turn: usize,
    past_events: Vec<GameAction>,
}

impl GameHistory {
    pub fn new(sender_id: String) -> Self {
        GameHistory {
            start_of_turn: 0,
            past_events: vec![],
        }
    }

    pub fn undo(&mut self) -> Option<GameAction>{
        if self.past_events.len() <= self.start_of_turn {
            return None;
        }

        Some(event.anti_event())
    }

    pub fn get_past_events(&self) -> &Vec<GameAction> {
        &self.past_events
    }
}

impl EventConsumer for EventBroker {
    fn handle_event(&mut self, event: &GameAction) {
        self.past_events.push(event.clone());

        if let GameAction::FinishTurn(_) = event {
            self.start_of_turn = self.past_events.len();
        }
    }
}
