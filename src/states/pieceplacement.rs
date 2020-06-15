use amethyst::{
    core::{
        transform::Transform,
        math::{Vector3, Point2},
    },
    input::{is_close_requested, is_key_down, VirtualKeyCode},
    prelude::*,
    renderer::{SpriteRender, resources::Tint},
    ui::{UiText, UiFinder, UiEventType, UiEvent},
    ecs::{WriteStorage, ReadStorage, ReadExpect, WriteExpect, Entity, Entities, Join},
};

use crate::{
    components::{
        Activatable, Piece, Cell,
        board::{BoardEvent, Team, Move, Range, Direction, BoardPosition, Target, TurnInto, Dying, PieceKind, TeamAssignment, Exhausted, ActivatablePower, Power},
    },
    states::{
        load::{UiElements,Sprites},
        next_turn::NextTurnState,
        PieceMovementState,
    },
    resources::board::{Board, Pattern, PatternComponent},
};
use crate::components::board::{Effect, EffectKind};

pub struct PiecePlacementState {

}

impl PiecePlacementState {
    pub fn new() -> PiecePlacementState {
        PiecePlacementState {
        }
    }

    pub fn update_ui((mut ui_text, ui_elements): (WriteStorage<UiText>, ReadExpect<UiElements>)){
        if let Some(text) = ui_text.get_mut(ui_elements.current_state_text) {
            text.text = "Place your piece.".parse().unwrap();
        }
    }

    pub fn update_targets((pieces, cells, positions, moves, powers, effects, teams, mut targets, entities, board):(
        ReadStorage<Piece>,
        ReadStorage<Cell>,
        ReadStorage<BoardPosition>,
        ReadStorage<Move>,
        ReadStorage<ActivatablePower>,
        ReadStorage<Effect>,
        ReadStorage<TeamAssignment>,
        WriteStorage<Target>,
        Entities,
        WriteExpect<Board>,
    )){
        for (_cell, cell_pos, mut target) in (&cells, &positions, &mut targets).join() {
            target.clear();
        }



        for (movement, _piece, piece_pos, e) in (&moves, &pieces, &positions, &entities).join() {
            movement.range.for_each(piece_pos.coords.x, piece_pos.coords.y, &board, |x,y,cell| {
                let target = targets.get_mut(cell).unwrap();
                target.add(e);
                true
            });
        }

        for (effect, _piece, piece_pos, e) in (&effects, &pieces, &positions, &entities).join() {
            effect.range.for_each(piece_pos.coords.x, piece_pos.coords.y, &board, |x,y,cell| {
                let target = targets.get_mut(cell).unwrap();
                if effect.kind == EffectKind::Protection {
                    target.protected = true;
                }
                true
            });
        }

        for (power, _piece, piece_pos, team, e) in (&powers, &pieces, &positions, &teams, &entities).join() {
            power.range.for_each(piece_pos.coords.x, piece_pos.coords.y, &board, |x,y,cell| {
                let target = targets.get_mut(cell).unwrap();

                if target.protected {
                    return false;
                } else if let Some(target_piece) = board.get_piece(x,y) {
                    if teams.get(target_piece).unwrap().id != team.id {
                        target.add_special(e);
                    }
                }

                true
            });
        }

    }

    pub fn merge_piece_patterns((mut board, mut teams, mut turn_intos, mut pieces, mut positions, mut dyings, entities):(
        WriteExpect<Board>,
        WriteStorage<TeamAssignment>,
        WriteStorage<TurnInto>,
        WriteStorage<Piece>,
        WriteStorage<BoardPosition>,
        WriteStorage<Dying>,
        Entities
    )){

        for pattern in &Pattern::all_patterns() {
        for x in 0..board.w as usize - pattern.components[0].len() + 1 {
            for y in 0..board.h as usize - pattern.components.len() + 1 {
                if let Some(mut matched_entities) = board.match_pattern(&pattern, x as u8, y as u8) {
                    let any_entities_team = teams.get(matched_entities[0]).unwrap();
                    println!("Pattern matched!");
                    if matched_entities.iter().all(|&x|
                        teams.get(x).unwrap().id == any_entities_team.id
                            && !turn_intos.contains(x)
                            && !dyings.contains(x)) {
                        matched_entities.iter_mut().for_each(|&mut matched_piece| {
                            dyings.insert(matched_piece, Dying {});
                            let pos = positions.get(matched_piece).unwrap();
                            board.remove_piece(pos.coords.x,pos.coords.y);
                        });
                        let new_piece = entities.create();
                        turn_intos.insert(new_piece, TurnInto { kind: pattern.turn_into });

                        let new_piece_x = x as u8 + pattern.new_piece_relative_position.coords.x;
                        let new_piece_y = y as u8 + pattern.new_piece_relative_position.coords.y;
                        positions.insert(new_piece, BoardPosition { coords: Point2::new(new_piece_x, new_piece_y) });

                        teams.insert(new_piece, *any_entities_team);
                        pieces.insert(new_piece, Piece::new());

                        println!("Matched pattern at {}:{}; new piece at {}:{}", x, y, new_piece_x, new_piece_y);
                    }
                }
            }
        }
        }
    }

    pub fn init_new_pieces(( mut board, sprites, mut moves, mut activatable_powers, mut pieces, mut positions,
                             mut turn_intos, mut teams, mut sprite_renders, mut tints, mut exhausted, mut effects, entities):
                         (
                          WriteExpect<Board>,
                          ReadExpect<Sprites>,
                          WriteStorage<Move>,
                          WriteStorage<ActivatablePower>,
                          WriteStorage<Piece>,
                          WriteStorage<BoardPosition>,
                          WriteStorage<TurnInto>,
                          WriteStorage<TeamAssignment>,
                          WriteStorage<SpriteRender>,
                          WriteStorage<Tint>,
                          WriteStorage<Exhausted>,
                          WriteStorage<Effect>,
                          Entities,
                         )) {


        for (turn_into, pos, team, _piece, e) in (&mut turn_intos, &positions, &teams, &pieces, &*entities).join() {
            board.place_piece(e, pos.coords.x, pos.coords.y);

            tints.insert(e, Tint(board.get_team(team).color));

            match turn_into.kind {
                PieceKind::HorizontalBar => {
                    moves.insert(e, Move::new(Direction::Horizontal, 255));
                    sprite_renders.insert(e, sprites.sprite_horizontal_bar.clone());
                    exhausted.insert(e, Exhausted{});
                    activatable_powers.insert(e, ActivatablePower{
                        range: Range::new_unlimited(Direction::Horizontal),
                        kind: Power::Blast,
                    });
                }
                PieceKind::VerticalBar => {
                    moves.insert(e, Move::new(Direction::Vertical, 255));
                    sprite_renders.insert(e, sprites.sprite_vertical_bar.clone());
                    exhausted.insert(e, Exhausted{});
                    activatable_powers.insert(e, ActivatablePower{
                        range: Range::new_unlimited(Direction::Vertical),
                        kind: Power::Blast,
                    });
                }
                PieceKind::Cross => {
                    moves.insert(e, Move::new(Direction::Straight, 255));
                    sprite_renders.insert(e, sprites.sprite_cross.clone());
                }
                PieceKind::Simple => {
                    moves.insert(e, Move::new(Direction::Star, 1));
                    sprite_renders.insert(e, sprites.sprite_piece.clone());
                }
                PieceKind::Queen => {
                    moves.insert(e, Move::new(Direction::Star, 255));
                    sprite_renders.insert(e, sprites.sprite_queen.clone());
                    exhausted.insert(e, Exhausted{});
                    activatable_powers.insert(e, ActivatablePower{
                        range: Range::new(Direction::Star, 1),
                        kind: Power::Blast,
                    });
                }
                PieceKind::Castle => {
                    sprite_renders.insert(e, sprites.sprite_protect.clone());
                    effects.insert(e, Effect{
                        kind: EffectKind::Protection,
                        range: Range {
                            direction: Direction::Star,
                            steps: 1,
                            jumps: true,
                            include_self: true
                        }
                    });
                }
                PieceKind::Sniper => {
                    sprite_renders.insert(e, sprites.sprite_sniper.clone());
                    exhausted.insert(e, Exhausted{});
                    activatable_powers.insert(e, ActivatablePower{
                        range: Range::anywhere(),
                        kind: Power::TargetedShoot,
                    });
                }
            }
        }

        turn_intos.clear();
     }
}

impl SimpleState for PiecePlacementState {

    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        data.world.exec(PiecePlacementState::update_ui);
        data.world.exec(PiecePlacementState::init_new_pieces);
        data.world.exec(PiecePlacementState::merge_piece_patterns);
        data.world.exec(PiecePlacementState::init_new_pieces);
        data.world.exec(PiecePlacementState::update_targets);
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
                    let mut teams = data.world.write_storage::<TeamAssignment>();

                    if let Some(piece) = board.get_piece(x, y) {
                        let exhausted = data.world.read_storage::<Exhausted>();
                        if teams.get(piece).unwrap().id == board.current_team().id && !exhausted.contains(piece){
                            println!("Moving piece");
                            Trans::Replace(Box::new(PieceMovementState { from_x: x, from_y: y, piece }))
                        } else {
                            Trans::None
                        }
                    } else {

                        let mut positions = data.world.write_storage::<BoardPosition>();
                        let mut turn_intos = data.world.write_storage::<TurnInto>();
                        let mut exhausted = data.world.write_storage::<Exhausted>();

                        if let Some(piece) = board.get_unused_piece() {
                            println!("Placed new piece");
                            positions.insert(piece, BoardPosition::new(x,y));
                            turn_intos.insert(piece, TurnInto{kind: PieceKind::Simple});
                            exhausted.insert(piece, Exhausted{});
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
