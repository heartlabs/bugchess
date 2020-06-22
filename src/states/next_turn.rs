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
        Activatable, Piece,
        board::{BoardEvent, Team},
    },
    states::{
        load::Sprites,
        PieceMovementState,
    }
};
use crate::states::load::UiElements;
use crate::components::board::{Move, Range, Direction, BoardPosition, Target, TurnInto, PieceKind};
use crate::components::{Cell, Bounded};
use crate::resources::board::{Board, Pattern, PatternComponent};
use crate::components::board::PieceKind::{HorizontalBar, Simple};
use crate::states::PiecePlacementState;
use crate::states::game_over::GameOverState;

pub struct TurnCounter {
    pub num_turns: u32,
}

pub struct NextTurnState {
    first_turn: bool,
    no_teams_left: bool,
}

impl NextTurnState {
    pub fn new() -> NextTurnState {
        NextTurnState {
            first_turn: false,
            no_teams_left: false
        }
    }

    pub fn first() -> NextTurnState {
        NextTurnState {
            first_turn: true,
            no_teams_left: false
        }
    }

    fn new_unused_pieces((pieces, board_positions, mut transforms, mut sprite_renders, mut board, sprites, entities): (
        ReadStorage<Piece>,
        ReadStorage<BoardPosition>,
        WriteStorage<Transform>,
        WriteStorage<SpriteRender>,
        WriteExpect<Board>,
        WriteExpect<Sprites>,
        Entities,
    )) {
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

    fn identify_losing_teams((pieces, board_positions, mut board, entities): (
        ReadStorage<Piece>,
        ReadStorage<BoardPosition>,
        WriteExpect<Board>,
        Entities,
    )) {
        let mut team_index = Vec::new();

        for _ in 0..board.num_teams() {
            team_index.push(false);
        }

        for (piece, b) in (&pieces, &board_positions).join() {
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

impl SimpleState for NextTurnState {

    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        data.world.exec(NextTurnState::identify_losing_teams);
        data.world.exec(|mut pieces: WriteStorage<Piece>| for mut p in (&mut pieces).join() {p.exhausted = false;});

        let team = {
            let mut board = data.world.write_resource::<Board>();
            if !self.first_turn {

                if board.next_team().is_none() {
                    self.no_teams_left = true;
                    return;
                }
            }

            board.current_team()
        };

        let new_pieces_per_turn = 2;

        for _ in 0..new_pieces_per_turn {
            data.world
                .create_entity()
                .with(Piece::new(team.id))
                .with(Tint(team.color))
                .build();
        }

        {
            let mut turn_counter = data.world.write_resource::<TurnCounter>();
            turn_counter.num_turns += 1;
        }

        data.world.exec(NextTurnState::new_unused_pieces);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans  {
        let board = data.world.write_resource::<Board>();

        {
            let mut ui_text = data.world.write_storage::<UiText>();
            let team = board.current_team();
            let ui_elements = data.world.read_resource::<UiElements>();
            if let Some(text) = ui_text.get_mut(ui_elements.current_team_text) {
                text.text = format!("Current Team: {}", team.name);
                text.color = [team.color.red, team.color.green, team.color.blue, team.color.alpha];
            }
        }

        if self.no_teams_left || board.num_unused_pieces() > 20 {
            return Trans::Replace(Box::new(GameOverState::new(board.current_team())))
        }

        Trans::Replace(Box::new(PiecePlacementState::new()))
    }
}
