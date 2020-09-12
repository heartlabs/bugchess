use amethyst::{
    core::{
        math::{Point2},
    },
    input::{is_close_requested, is_key_down, VirtualKeyCode},
    prelude::*,
    renderer::{SpriteRender, resources::Tint},
    ui::{UiText, UiEventType, UiEvent},
    ecs::{WriteStorage, ReadStorage, ReadExpect, WriteExpect, Entities, Join, RunNow},
};

use crate::{
    components::{
        Cell,
        board::{BoardEvent, BoardPosition, Target, },
        piece::{Piece, Move, Range, Direction, TurnInto, PieceKind, ActivatablePower, Power, Effect, EffectKind, Exhaustion, ExhaustionStrategy},
    },
    states::{
        load::{UiElements,Sprites},
        next_turn::NextTurnState,
        PieceMovementState,
    },
    resources::board::{Board, Pattern},
};
use crate::systems::actions::common::UpdateUi;
use crate::systems::actions::during_turn::{InitNewPieces, MergePiecePatterns, UpdateTargets};
use crate::states::load::Actions;
use std::borrow::BorrowMut;
use std::ops::Deref;
use crate::systems::actions::actions::{HasRunNow, AddUnusedPiece, ProducesActions};
use crate::systems::actions::place::Place;

pub struct PiecePlacementState {

}

impl PiecePlacementState {
    pub fn new () -> PiecePlacementState {
        PiecePlacementState{

        }
    }
}


impl SimpleState for PiecePlacementState {

    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        {
            let mut actions = data.world.write_resource::<Actions>();
            actions.run_queue(&data.world);
        }

        data.world.maintain(); // This makes sure that deleted entities are actually deleted

        let local_actions: Vec<Box<dyn HasRunNow>> = vec![
            Box::new(UpdateUi{text: "Place your Piece"}),
            Box::new(InitNewPieces{}),
            MergePiecePatterns::new(),
            Box::new(InitNewPieces{}),
            Box::new(UpdateTargets{}),
        ];

        for action in local_actions {
            action.get_run_now().run_now(&data.world);

            let mut actions = data.world.write_resource::<Actions>();
            actions.run_queue(&data.world);
        }

        let mut actions = data.world.write_resource::<Actions>();
        actions.finalize_player_move();

    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        match &event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                    return Trans::Quit
                }

                if is_key_down(&event, VirtualKeyCode::U) {
                    data.world.write_resource::<Actions>().undo(data.world);
                    return Trans::Replace(Box::new(PiecePlacementState::new()));
                }
            }
            StateEvent::Ui(UiEvent{target: _, event_type: UiEventType::ClickStart}) => {
            }
            StateEvent::Input(_input) => {
            }
            _ => {}
        }
        Trans::None
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans  {

        let mut board = data.world.write_resource::<Board>();

        if let Some(event) = board.poll_event() {
            match event {
                BoardEvent::Cell { x, y } => {
                    println!("Cell Event {},{}", x, y);
                    let mut pieces = data.world.write_storage::<Piece>();

                    if let Some(piece) = board.get_piece(x, y) {
                        let piece_component = pieces.get_mut(piece).unwrap();
                        if piece_component.team_id == board.current_team().id && !piece_component.exhaustion.is_done() {
                            println!("Moving piece");
                            Trans::Replace(Box::new(PieceMovementState { from_x: x, from_y: y, piece }))
                        } else {
                            Trans::None
                        }
                    } else {

                        if let Some(piece) = board.get_unused_piece() {
                            let mut actions = data.world.write_resource::<Actions>();

                            // Even though we don't run the remove action directly, we just did what it would do.
                            // So we add it to the history so that it will be undone on undoing the current move
                            actions.insert_only(Box::new(AddUnusedPiece::remove(piece)));

                            actions.add_to_queue(Place::introduce(
                                piece,
                                BoardPosition::new(x,y),
                                PieceKind::Simple
                            ));
                            Trans::Replace(Box::new(PiecePlacementState::new()))
                        } else {
                            Trans::None
                        }

                    }
                },
                BoardEvent::Next => {
                    Trans::Replace(Box::new(NextTurnState::new()))
                },
                _ => Trans::None
            }
        } else {
            Trans::None
        }
    }
}
