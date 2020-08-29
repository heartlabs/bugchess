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
        data.world.maintain(); // This makes sure that deleted entities are actually deleted
        let actions = data.world.read_resource::<Actions>();
        for action in actions.on_start.as_slice() {
            action.get_run_now().run_now(&data.world);
        }
    }

    fn handle_event(
        &mut self,
        _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        match &event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                    return Trans::Quit
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
                        let mut positions = data.world.write_storage::<BoardPosition>();
                        let mut turn_intos = data.world.write_storage::<TurnInto>();

                        if let Some(piece) = board.get_unused_piece() {
                            let piece_component = pieces.get_mut(piece).unwrap();
                            piece_component.pierce = false;

                            println!("Placed new piece");
                            positions.insert(piece, BoardPosition::new(x,y));
                            turn_intos.insert(piece, TurnInto{kind: PieceKind::Simple});
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
