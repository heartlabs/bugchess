use amethyst::{
    renderer::{
        resources::Tint,
        palette::Srgba,
    },
    ecs::{Join, ReadStorage, System, WriteStorage, WriteExpect, Entities},
};
use crate::components::{Activatable, Piece};
use crate::components::active::{Selected, Hovered};
use crate::components::board::{BoardPosition};
use crate::resources::board::Board;

pub struct DyingSystem;

impl<'a> System<'a> for DyingSystem {
    type SystemData = (
        ReadStorage<'a, Piece>,
        ReadStorage<'a, BoardPosition>,
        WriteExpect<'a, Board>,
        Entities<'a>,
    );

    fn run(&mut self, (pieces, board_positions, mut board, entities): Self::SystemData) {
        for (piece, pos, e) in (&pieces, &board_positions, &*entities).join() {
            if !piece.dying {
                continue;
            }

            entities.delete(e);
            if let Some(piece_at_pos) = board.get_piece(pos.coords.x, pos.coords.y) {
                if piece_at_pos == e {
                    board.remove_piece(pos.coords.x, pos.coords.y);
                }
            }
        }
    }
}
