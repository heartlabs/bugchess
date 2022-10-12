use crate::{
    actions::{
        compound_events::{CompoundEventBuilder, GameAction, FlushResult},
        merge::MergeBuilder,
    },
    atomic_events::AtomicEvent::{self, *},
    game_events::{EventConsumer, GameEventObject},
};
use game_model::game::Game;
use miniquad::warn;
use std::{cell::RefCell, rc::Rc};

pub struct BoardEventConsumer {
    pub own_sender_id: String,
    pub(crate) game: Rc<RefCell<Box<Game>>>,
}

impl EventConsumer for BoardEventConsumer {
    fn handle_event(&mut self, event_object: &GameEventObject) {
        if !matches!(event_object.event, GameAction::Undo(_))
            && event_object.sender == self.own_sender_id
        {
            return;
        }

        let event = &event_object.event;
        println!("Handling event {:?}", event);

        event
            .get_compound_event()
            .get_events()
            .iter()
            .for_each(|e| {
                BoardEventConsumer::handle_event_internal((*self.game).borrow_mut().as_mut(), e)
            });
    }
}

impl BoardEventConsumer {
    pub fn new(own_sender_id: String, game: Rc<RefCell<Box<Game>>>) -> Self {
        BoardEventConsumer {
            own_sender_id,
            game,
        }
    }

    pub fn flush_unsafe(game: &mut Game, action: &GameAction) {
        for event in action.get_compound_event().get_events() {
            BoardEventConsumer::handle_event_internal(game, &event);
        }
    }

    pub fn flush(game: &mut Game, action: Box<dyn CompoundEventBuilder>) -> FlushResult {
        action.flush(&mut |event| {
            BoardEventConsumer::handle_event_internal(game, &event);
        })
    }

    fn handle_event_internal(game: &mut Game, event: &AtomicEvent) {
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
                let piece = &mut board.get_piece_mut_at(point).expect(&*format!(
                    "Can't execute {:?} for non-existing piece at {:?}",
                    event, point
                ));

                assert_eq!(
                    from, &piece.exhaustion,
                    "Expected piece at {:?} to have exhaustion state {:?} but it had {:?}",
                    point, from, piece.exhaustion
                );

                piece.exhaustion = *to;
            }
            AddEffect(kind, at) => {
                board.add_effect(*kind, at);
            }
            RemoveEffect(kind, at) => board.remove_effect(kind, at),
            NextTurn => {
                warn!("NEXT TURN");
                game.next_team();
            }
        }
    }
}
