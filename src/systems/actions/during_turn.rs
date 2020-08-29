use amethyst::{
    ecs::{System, WriteStorage, ReadStorage, ReadExpect, WriteExpect, Entities, Join},
    core::math::{Point2},
    renderer::{SpriteRender, resources::Tint},
};
use crate::{
    components::{
        piece::*,
        board::{Target, BoardPosition},
        Cell,
    },
    resources::board::{Board,Pattern},
    states::load::Sprites,
};
use crate::systems::actions::actions::HasRunNow;
use amethyst::core::ecs::RunNow;

pub struct UpdateTargets {}

impl HasRunNow for UpdateTargets {
    fn get_run_now<'a>(&self) -> Box<dyn RunNow<'a>> {
        Box::new(Self{})
    }
}

impl<'a> System<'a> for UpdateTargets {
    type SystemData = (
        ReadStorage<'a, Piece>,
        ReadStorage<'a, Cell>,
        ReadStorage<'a, BoardPosition>,
        ReadStorage<'a, Effect>,
        WriteStorage<'a, Target>,
        Entities<'a>,
        WriteExpect<'a, Board>,
    );

    fn run(&mut self, (pieces, cells, positions, effects, mut targets, entities, board): Self::SystemData) {
        for (_cell, _cell_pos, target) in (&cells, &positions, &mut targets).join() {
            target.clear();
        }

        for (piece, piece_pos, e) in (&pieces, &positions, &entities).join() {
            if let Some(movement) = &piece.movement {
                movement.range.for_each(piece_pos.coords.x, piece_pos.coords.y, &board, |x, y, cell| {
                    let target = targets.get_mut(cell).unwrap();
                    target.add(e);
                    true
                });
            }
        }

        for (effect, _piece, piece_pos, e) in (&effects, &pieces, &positions, &entities).join() {
            effect.range.for_each(piece_pos.coords.x, piece_pos.coords.y, &board, |x, y, cell| {
                let target = targets.get_mut(cell).unwrap();

                let board_piece = board.get_piece(piece_pos.coords.x, piece_pos.coords.y);
                if board_piece.is_none() || board_piece.unwrap() != e {
                    println!("Something is rotten at {:?}", piece_pos);
                }
                if effect.kind == EffectKind::Protection {
                    target.protected = true;
                }
                true
            });
        }

        for (piece, piece_pos, e) in (&pieces, &positions, &entities).join() {
            if let Some(power) = piece.activatable.as_ref() {
                power.range.for_each(piece_pos.coords.x, piece_pos.coords.y, &board, |x, y, cell| {
                    let target = targets.get_mut(cell).unwrap();

                    if target.protected {
                        return false;
                    } else if let Some(target_piece) = board.get_piece(x, y) {
                        if pieces.get(target_piece).unwrap().team_id != piece.team_id {
                            target.add_special(e);
                        }
                    }

                    true
                });
            }
        }
    }
}

pub struct MergePiecePatterns {}

impl HasRunNow for MergePiecePatterns {
    fn get_run_now<'a>(&self) -> Box<dyn RunNow<'a>> {
        Box::new(Self{})
    }
}

impl<'a> System<'a> for MergePiecePatterns {
    type SystemData = (
        WriteExpect<'a, Board>,
        WriteStorage<'a, TurnInto>,
        WriteStorage<'a, Piece>,
        WriteStorage<'a, BoardPosition>,
        Entities<'a>
    );

    fn run(&mut self, (mut board, mut turn_intos, mut pieces, mut positions, entities): Self::SystemData) {
        for pattern in &Pattern::all_patterns() {
            for x in 0..board.w as usize - pattern.components[0].len() + 1 {
                for y in 0..board.h as usize - pattern.components.len() + 1 {
                    if let Some(mut matched_entities) = board.match_pattern(&pattern, x as u8, y as u8) {
                        let any_team_id = { pieces.get(matched_entities[0]).unwrap().team_id };
                        println!("Pattern matched!");
                        if matched_entities.iter().all(|&x|
                            pieces.get(x).unwrap().team_id == any_team_id
                                && !turn_intos.contains(x)
                                && !pieces.get(x).unwrap().dying) {
                            matched_entities.iter_mut().for_each(|&mut matched_piece| {
                                println!("Going to remove matched piece {:?}", matched_piece);
                                pieces.get_mut(matched_piece).unwrap().dying = true;
                                let pos = positions.get(matched_piece).unwrap();
                                board.remove_piece(pos.coords.x, pos.coords.y);
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
}

pub struct InitNewPieces {}

impl HasRunNow for InitNewPieces {
    fn get_run_now<'a>(&self) -> Box<dyn RunNow<'a>> {
        Box::new(Self{})
    }
}

impl<'a> System<'a> for InitNewPieces {
    type SystemData = (
        WriteExpect<'a, Board>,
        ReadExpect<'a, Sprites>,
        WriteStorage<'a, Piece>,
        ReadStorage<'a, BoardPosition>,
        WriteStorage<'a, TurnInto>,
        WriteStorage<'a, SpriteRender>,
        WriteStorage<'a, Tint>,
        WriteStorage<'a, Effect>,
        Entities<'a>,
    );

    fn run(&mut self, (mut board, sprites, mut pieces, positions,
        mut turn_intos, mut sprite_renders, mut tints, mut effects, entities): Self::SystemData) {
        for (turn_into, pos, mut piece, e) in (&mut turn_intos, &positions, &mut pieces, &*entities).join() {
            board.place_piece(e, pos.coords.x, pos.coords.y);

            tints.insert(e, Tint(board.get_team(piece.team_id).color));

            match turn_into.kind {
                PieceKind::HorizontalBar => {
                    piece.movement = Some(Move::new(Direction::Horizontal, 255));
                    sprite_renders.insert(e, sprites.sprite_horizontal_bar.clone());
                    piece.activatable = Some(ActivatablePower {
                        range: Range::new_unlimited(Direction::Horizontal),
                        kind: Power::Blast,
                    });
                }
                PieceKind::VerticalBar => {
                    piece.movement = Some(Move::new(Direction::Vertical, 255));
                    sprite_renders.insert(e, sprites.sprite_vertical_bar.clone());
                    piece.activatable = Some(ActivatablePower {
                        range: Range::new_unlimited(Direction::Vertical),
                        kind: Power::Blast,
                    });
                }
                PieceKind::Cross => {
                    piece.movement = Some(Move::new(Direction::Straight, 255));
                    piece.shield = true;
                    sprite_renders.insert(e, sprites.sprite_cross.clone());
                }
                PieceKind::Simple => {
                    piece.movement = Some(Move::new(Direction::Star, 1));
                    sprite_renders.insert(e, sprites.sprite_piece.clone());
                }
                PieceKind::Queen => {
                    piece.movement = Some(Move::new(Direction::Star, 255));
                    sprite_renders.insert(e, sprites.sprite_queen.clone());
                    piece.exhaustion = Exhaustion::new_exhausted(ExhaustionStrategy::Both);
                    piece.activatable = Some(ActivatablePower {
                        range: Range::new(Direction::Star, 1),
                        kind: Power::Blast,
                    });
                }
                PieceKind::Castle => {
                    sprite_renders.insert(e, sprites.sprite_protect.clone());
                    effects.insert(e, Effect {
                        kind: EffectKind::Protection,
                        range: Range {
                            direction: Direction::Star,
                            steps: 1,
                            jumps: true,
                            include_self: true,
                        },
                    });
                }
                PieceKind::Sniper => {
                    sprite_renders.insert(e, sprites.sprite_sniper.clone());
                    piece.activatable = Some(ActivatablePower {
                        range: Range::anywhere(),
                        kind: Power::TargetedShoot,
                    });
                }
            }
        }

        turn_intos.clear();
    }
}
