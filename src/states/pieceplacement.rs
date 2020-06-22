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
        board::{BoardEvent, Team, Move, Range, Direction, BoardPosition, Target, TurnInto, PieceKind, ActivatablePower, Power},
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

    pub fn update_targets((pieces, cells, positions, moves, powers, effects, mut targets, entities, board):(
        ReadStorage<Piece>,
        ReadStorage<Cell>,
        ReadStorage<BoardPosition>,
        ReadStorage<Move>,
        ReadStorage<ActivatablePower>,
        ReadStorage<Effect>,
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

                let board_piece = board.get_piece(piece_pos.coords.x, piece_pos.coords.y);
                if (board_piece.is_none() || board_piece.unwrap() != e){
                    println!("Something is rotten at {:?}", piece_pos);
                }
                if effect.kind == EffectKind::Protection {
                    target.protected = true;
                }
                true
            });
        }

        for (power, piece, piece_pos, e) in (&powers, &pieces, &positions, &entities).join() {
            power.range.for_each(piece_pos.coords.x, piece_pos.coords.y, &board, |x,y,cell| {
                let target = targets.get_mut(cell).unwrap();

                if target.protected {
                    return false;
                } else if let Some(target_piece) = board.get_piece(x,y) {
                    if pieces.get(target_piece).unwrap().team_id != piece.team_id {
                        target.add_special(e);
                    }
                }

                true
            });
        }

    }

    // pub fn clean_up_dead_pieces((dyings, board_positions, mut board, entities):
    // (
    //     ReadStorage<Dying>,
    //     ReadStorage<BoardPosition>,
    //     WriteExpect<Board>,
    //     Entities,
    // )) {
    //     for (_dying, pos, e) in (&dyings, &board_positions, &*entities).join() {
    //         entities.delete(e);
    //         if let Some(piece_at_pos) = board.get_piece(pos.coords.x, pos.coords.y) {
    //             if piece_at_pos == e {
    //                 board.remove_piece(pos.coords.x, pos.coords.y);
    //             }
    //         }
    //     }
    // }

    pub fn merge_piece_patterns((mut board, mut turn_intos, mut pieces, mut positions, entities):(
        WriteExpect<Board>,
        WriteStorage<TurnInto>,
        WriteStorage<Piece>,
        WriteStorage<BoardPosition>,
        Entities
    )){

        for pattern in &Pattern::all_patterns() {
        for x in 0..board.w as usize - pattern.components[0].len() + 1 {
            for y in 0..board.h as usize - pattern.components.len() + 1 {
                if let Some(mut matched_entities) = board.match_pattern(&pattern, x as u8, y as u8) {
                    let any_team_id = {pieces.get(matched_entities[0]).unwrap().team_id};
                    println!("Pattern matched!");
                    if matched_entities.iter().all(|&x|
                        pieces.get(x).unwrap().team_id == any_team_id
                            && !turn_intos.contains(x)
                            && !pieces.get(x).unwrap().dying) {
                        matched_entities.iter_mut().for_each(|&mut matched_piece| {
                            println!("Going to remove matched piece {:?}", matched_piece);
                            pieces.get_mut(matched_piece).unwrap().dying = true;
                            let pos = positions.get(matched_piece).unwrap();
                            board.remove_piece(pos.coords.x,pos.coords.y);
                        });
                        let new_piece = entities.create();
                        turn_intos.insert(new_piece, TurnInto { kind: pattern.turn_into });

                        let new_piece_x = x as u8 + pattern.new_piece_relative_position.coords.x;
                        let new_piece_y = y as u8 + pattern.new_piece_relative_position.coords.y;
                        positions.insert(new_piece, BoardPosition { coords: Point2::new(new_piece_x, new_piece_y) });

                        pieces.insert(new_piece, Piece::new(any_team_id));

                        println!("Matched pattern at {}:{}; new piece at {}:{}", x, y, new_piece_x, new_piece_y);
                    }
                }
            }
        }
        }
    }

    pub fn init_new_pieces(( mut board, sprites, mut moves, mut activatable_powers, mut pieces, mut positions,
                             mut turn_intos, mut sprite_renders, mut tints, mut effects, entities):
                         (
                          WriteExpect<Board>,
                          ReadExpect<Sprites>,
                          WriteStorage<Move>,
                          WriteStorage<ActivatablePower>,
                          WriteStorage<Piece>,
                          WriteStorage<BoardPosition>,
                          WriteStorage<TurnInto>,
                          WriteStorage<SpriteRender>,
                          WriteStorage<Tint>,
                          WriteStorage<Effect>,
                          Entities,
                         )) {


        for (turn_into, pos, mut piece, e) in (&mut turn_intos, &positions, &mut pieces, &*entities).join() {
            board.place_piece(e, pos.coords.x, pos.coords.y);

            tints.insert(e, Tint(board.get_team(piece.team_id).color));

            match turn_into.kind {
                PieceKind::HorizontalBar => {
                    moves.insert(e, Move::new(Direction::Horizontal, 255));
                    sprite_renders.insert(e, sprites.sprite_horizontal_bar.clone());
                    piece.exhausted = true;
                    activatable_powers.insert(e, ActivatablePower{
                        range: Range::new_unlimited(Direction::Horizontal),
                        kind: Power::Blast,
                    });
                }
                PieceKind::VerticalBar => {
                    moves.insert(e, Move::new(Direction::Vertical, 255));
                    sprite_renders.insert(e, sprites.sprite_vertical_bar.clone());
                    piece.exhausted = true;
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
                    piece.exhausted = true;
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
                    piece.exhausted = true;
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
        data.world.maintain(); // This makes sure that deleted entities are actually deleted
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
                    let mut pieces = data.world.write_storage::<Piece>();

                    if let Some(piece) = board.get_piece(x, y) {
                        let piece_component = pieces.get_mut(piece).unwrap();
                        if piece_component.team_id == board.current_team().id && !piece_component.exhausted {
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

                            println!("Placed new piece");
                            positions.insert(piece, BoardPosition::new(x,y));
                            turn_intos.insert(piece, TurnInto{kind: PieceKind::Simple});
                            piece_component.exhausted = true;
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
