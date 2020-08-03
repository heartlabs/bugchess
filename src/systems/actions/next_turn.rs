use amethyst::{
    core::{
        transform::Transform,
        math::Vector3,
    },
    renderer::{SpriteRender, resources::Tint},
    ecs::{WriteStorage, ReadStorage, WriteExpect, Entities, Join, System, RunNow},
};

use crate::{
    components::{
        board::{BoardPosition,},
        piece::{Piece, }
    },
    resources::board::Board,
    states::{
        load::Sprites,
        next_turn::NextTurnState
    },
};

pub(crate) struct IdentifyLosingTeams {}

impl<'a, 'b> System<'a> for NextTeam<'b> {
    type SystemData = (
        WriteStorage<'a, Tint>,
        WriteStorage<'a, Piece>,
        ReadStorage<'a, BoardPosition>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, SpriteRender>,
        WriteExpect<'a, Board>,
        WriteExpect<'a, Sprites>,
        Entities<'a>);

    fn run(&mut self, (mut tints, mut pieces, board_positions, mut transforms, mut sprite_renders, mut board, sprites, entities): Self::SystemData) {
        for mut p in (&mut pieces).join() {
            p.exhaustion.reset();
        }

        let team = {
            if !self.state.first_turn {

                if board.next_team().is_none() {
                    self.state.no_teams_left = true;
                    return;
                }
            }

            board.current_team()
        };

        let new_pieces_per_turn = 2;

        for _ in 0..new_pieces_per_turn {
            entities.build_entity()
                .with(Piece::new(team.id), &mut pieces)
                .with(Tint(team.color), &mut tints)
                .build();
        }

        for (piece, _b, e) in (&pieces, !&board_positions, &*entities).join() {
            if piece.team_id == board.current_team().id && !transforms.contains(e){

                let mut transform = Transform::default();

                // 2 rows with 10 pieces each per team
                let row: usize = piece.team_id * 2 + board.num_unused_pieces()/10;
                let column: usize = board.num_unused_pieces()%10;

                let x_offset = 650;
                let y_offset = 100;

                let piece_width = 32;

                let screen_x = (x_offset + column * piece_width) as f32;
                let screen_y = (y_offset + row * piece_width) as f32;
                //println!("New unused piece at {}:{}", screen_x, screen_y);

                transform.set_translation_xyz(screen_x, screen_y, 0.1);
                transform.set_scale(Vector3::new(0.5,0.5,1.));
                transforms.insert(e, transform);

                sprite_renders.insert(e, sprites.sprite_piece.clone());

                board.add_unused_piece(e);
            }
        }
    }

}

impl<'a> System<'a> for IdentifyLosingTeams {
    type SystemData = (
        ReadStorage<'a, Piece>,
        ReadStorage<'a, BoardPosition>,
        WriteExpect<'a, Board>,
    );
    fn run(&mut self, (pieces, board_positions, mut board): Self::SystemData) {
        let mut team_index = Vec::new();

        for _ in 0..board.num_teams() {
            team_index.push(false);
        }

        for (piece, _b) in (&pieces, &board_positions).join() {
            team_index[piece.team_id] = true;
        }

        for (i, has_pieces_left) in team_index.iter().enumerate() {
            if !has_pieces_left {
                println!("team {} lost", i);
                board.mark_team_as_lost(i);
            }
        }
    }
}

pub struct NextTeam<'a> {
    pub(crate) state: &'a mut NextTurnState
}
