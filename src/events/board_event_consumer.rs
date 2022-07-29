use crate::{
    events::{
        compound_events::CompoundEvent,
        game_events::{EventConsumer, GameEventObject},
    },
    game_logic::game::Game,
    AtomicEvent,
    AtomicEvent::{
        AddEffect, AddUnusedPiece, ChangeExhaustion, NextTurn, Place, Remove, RemoveEffect,
        RemoveUnusedPiece,
    },
    GameAction,
};
use macroquad::logging::warn;
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
    pub fn flush(game: &mut Game, action: &mut GameAction) {
        for event in action.get_compound_event_mut().flush() {
            BoardEventConsumer::handle_event_internal(game, &event);
        }
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
