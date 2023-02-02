use game_events::{
    actions::compound_events::{CompoundEventBuilder, FlushResult, GameAction},
    atomic_events::AtomicEvent,
    event_broker::EventConsumer,
};
use game_model::{game::Game, GameError, GameResult};
use miniquad::{debug, warn};
use std::{cell::RefCell, rc::Rc};

pub struct BoardEventConsumer {
    pub game: Rc<RefCell<Game>>,
}

impl EventConsumer for BoardEventConsumer {
    fn handle_event(&mut self, event: &GameAction) {
        debug!("Handling event {}", event);

        event
            .get_compound_event()
            .get_events()
            .iter()
            .for_each(|e| {
                BoardEventConsumer::handle_event_internal(&mut (*self.game).borrow_mut(), e)
                    .unwrap_or_else(|e| panic!("Failed to handle board event: {:?}", e))
                // TODO: propagate error
            });
    }
}

impl BoardEventConsumer {
    pub fn new(game: Rc<RefCell<Game>>) -> Self {
        BoardEventConsumer { game }
    }

    pub fn flush(game: &mut Game, action: Box<dyn CompoundEventBuilder>) -> FlushResult {
        action.flush(&mut |event| {
            BoardEventConsumer::handle_event_internal(game, &event)
                .unwrap_or_else(|e| panic!("Failed to handle board event: {:?}", e));
            // TODO: propagate error
        })
    }

    fn handle_event_internal(game: &mut Game, event: &AtomicEvent) -> GameResult<()> {
        let board = &mut game.board;

        match event {
            AtomicEvent::Place(at, piece) => {
                board.place_piece_at(*piece, at)?;
            }
            AtomicEvent::Remove(at, _) => {
                board.remove_piece_at(at)?;
            }
            AtomicEvent::AddUnusedPiece(team_id) => {
                game.add_unused_piece_for(*team_id);
            }
            AtomicEvent::RemoveUnusedPiece(team_id) => {
                game.remove_unused_piece(*team_id);
            }
            AtomicEvent::ChangeExhaustion(from, to, point) => {
                let piece = &mut board.get_piece_mut_at(point).ok_or(GameError::new(format!(
                    "Can't execute {:?} for non-existing piece at {:?}",
                    event, point
                )))?;

                if from != &piece.exhaustion {
                    return Err(GameError::new(format!(
                        "Expected piece at {:?} to have exhaustion state {:?} but it had {:?}",
                        point, from, piece.exhaustion
                    )));
                }

                piece.exhaustion = *to;
            }
            AtomicEvent::AddEffect(kind, at) => {
                board.add_effect(*kind, at)?;
            }
            AtomicEvent::RemoveEffect(kind, at) => board.remove_effect(kind, at)?,
            AtomicEvent::NextTurn => {
                warn!("NEXT TURN");
                game.next_team();
            }
        }

        Ok(())
    }
}
