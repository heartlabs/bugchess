use amethyst::{
    core::{
        transform::Transform,
        math::{Vector3},
    },
    ui::UiText,
    ecs::{ReadStorage, WriteStorage, Entity},
    input::{is_close_requested, is_key_down, VirtualKeyCode},
    prelude::*,
};

use crate::{
    components::{Activatable, Piece,
                 active::Selected,
                 board::{BoardEvent, BoardPosition, Target, Team, Dying, TeamAssignment, Exhausted, ActivatablePower, Power},
    },
    states::{
        PiecePlacementState,
        load::UiElements,
        next_turn::NextTurnState,
    },
    resources::board::Board,
};

use log::info;
use crate::states::target_for_power::TargetForPowerState;

pub struct PieceMovementState {
    pub from_x: u8,
    pub from_y: u8,
    pub piece: Entity,
}

impl SimpleState for PieceMovementState {
    // On start will run when this state is initialized. For more
    // state lifecycle hooks, see:
    // https://book.amethyst.rs/stable/concepts/state.html#life-cycle
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        {
            let mut ui_text = data.world.write_storage::<UiText>();
            let ui_elements = data.world.read_resource::<UiElements>();
            if let Some(text) = ui_text.get_mut(ui_elements.current_state_text) {
                text.text = "Move your piece.".parse().unwrap();
            }
        }

        let mut selected = data.world.write_storage::<Selected>();
        let board = data.world.read_resource::<Board>();
        let cell = board.get_cell(self.from_x, self.from_y);

        selected.insert(cell, Selected{});

    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let mut selected = data.world.write_storage::<Selected>();
        let board = data.world.read_resource::<Board>();
        let cell = board.get_cell(self.from_x, self.from_y);

        selected.remove(cell);
    }

    fn handle_event(
        &mut self,
        mut _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        match &event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                    Trans::Quit
                } else {
                    Trans::None
                }
            }
            StateEvent::Ui(ui_event) => {
                info!(
                    "[HANDLE_EVENT] You just interacted with a ui element: {:?}",
                    ui_event
                );
                Trans::None
            }
            StateEvent::Input(_input) => {
                Trans::None
            }
        }


    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans  {
        let mut board = data.world.write_resource::<Board>();

        if let Some(event) = board.poll_event()
        {
            match event {
                BoardEvent::Next => {
                    return Trans::Replace(Box::new(NextTurnState::new()));
                },
                BoardEvent::Cell { x, y } => {
                    println!("Cell Event {},{}", x, y);

                    let piece_at_target = board.get_piece(x, y);

                    let mut dyings = data.world.write_storage::<Dying>();
                    let mut exhausted = data.world.write_storage::<Exhausted>();
                    let mut targets = data.world.write_storage::<Target>();

                    if let Some(new_piece) = piece_at_target {
                        let teams = data.world.read_storage::<TeamAssignment>();
                        if let Some(new_piece_team) = teams.get(new_piece) {
                            let activatable_powers = data.world.read_storage::<ActivatablePower>();
                            if new_piece_team.id == board.current_team().id {
                                return self.handle_own_piece_at_target(x, y, new_piece,
                                                 &mut board, &teams, &mut dyings, &mut exhausted, &targets, &activatable_powers);
                            }
                        }
                    }
                    let pieces = data.world.read_storage::<Piece>();
                    let self_piece_component = pieces.get(self.piece).unwrap();
                    let invalid_attack = piece_at_target.is_some() && !self_piece_component.attack;


                    let cell = board.get_cell(x,y);
                    let target = targets.get(cell).unwrap();

                    let invalid_target_cell = !target.is_possible_target_of(self.piece);

                    println!("target piece: {:?} ; invalid attack: {} ; invalid target cell: {}", piece_at_target, invalid_attack, invalid_target_cell);

                    if invalid_attack || invalid_target_cell {
                        return Trans::Replace(Box::new(PiecePlacementState::new()))
                    } else if let Some(attacked_piece) = piece_at_target {
                        dyings.insert(attacked_piece, Dying{});
                    }

                    let mut positions = data.world.write_storage::<BoardPosition>();

                    exhausted.insert(self.piece, Exhausted{});

                    let mut pos = positions.get_mut(self.piece).unwrap();

                    pos.coords.x = x;
                    pos.coords.y = y;

                    board.move_piece(self.piece, self.from_x, self.from_y, x, y);
                    return Trans::Replace(Box::new(PiecePlacementState::new()));

                },
                _ => { }
            }
        }

        Trans::None

    }
}

impl PieceMovementState {

    fn handle_own_piece_at_target(&mut self, x: u8, y: u8, piece_at_target: Entity,
                                  mut board: &mut Board,
                                  teams: &ReadStorage<TeamAssignment>,
                                  mut dyings: &mut WriteStorage<Dying>,
                                  mut exhausted: &mut WriteStorage<Exhausted>,
                                  targets: &WriteStorage<Target>,
                                  mut activatable_powers: &ReadStorage<ActivatablePower>) -> SimpleTrans{
        if self.piece == piece_at_target{
            if let Some(power) = activatable_powers.get(self.piece) {
                return match power.kind {
                    Power::Blast => {
                        self.activate_blast(x,y,power, &mut board, &teams, &mut dyings, &mut exhausted, targets);
                        Trans::Replace(Box::new(PiecePlacementState::new()))
                    },
                    Power::TargetedShoot => {
                        Trans::Replace(Box::new(TargetForPowerState {
                            from_x: self.from_x,
                            from_y: self.from_y,
                            piece: self.piece
                        }))
                    },
                }
            }
        }
        else {
            return Trans::Replace(Box::new(PieceMovementState { from_x: x, from_y: y, piece: piece_at_target }));
        }

        return Trans::None;
    }

    fn activate_blast(&mut self, x: u8, y: u8, power: &ActivatablePower,
                      board: &mut Board,
                      teams: &ReadStorage<TeamAssignment>,
                      dyings: &mut WriteStorage<Dying>,
                      exhausted: &mut WriteStorage<Exhausted>,
                      targets: &WriteStorage<Target>) {
        power.range.for_each(x,y, &board, |power_x, power_y, cell| {
            if targets.get(cell).unwrap().protected {
                return false;
            }

            if let Some(target_piece) = board.get_piece(power_x as u8, power_y as u8){
                let target_piece_team = teams.get(target_piece).unwrap();
                if target_piece_team.id != board.current_team().id {
                    dyings.insert(target_piece, Dying {});
                    exhausted.insert(self.piece, Exhausted{});
                }
            }

            return true;
        });
    }
}
