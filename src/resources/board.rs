use crate::{
    components::{
        board::{Team, BoardEvent, BOARD_WIDTH, BOARD_HEIGHT, BoardPosition},
        piece::{PieceKind}
    }
};
use amethyst::ecs::{Entity};
use amethyst::core::math::{Point2};

pub struct Board {
    cells: Vec<Vec<Entity>>,
    placed_pieces: Vec<Vec<Option<Entity>>>,
    unused_pieces: Vec<Vec<Entity>>,
    teams: Vec<Team>,
    current_team_index: usize,
    pub w: u8,
    pub h: u8,
    event: Option<BoardEvent>,
}

impl Board {
    pub fn new(cells: Vec<Vec<Entity>>, teams: Vec<Team>) -> Board {
        let pieces = (0..BOARD_WIDTH)
            .map(|_i| {
                (0..BOARD_HEIGHT)
                    .map(|_j| {
                        Option::None
                    })
                    .collect()
            })
            .collect();

        let unused_pieces = teams.iter().map(|_| Vec::new()).collect();

        Board {
            cells,
            placed_pieces: pieces,
            unused_pieces,
            teams,
            current_team_index: 0,
            w: BOARD_WIDTH,
            h: BOARD_HEIGHT,
            event: Option::None,
        }
    }

    pub fn add_unused_piece(&mut self, piece: Entity) {
        self.unused_pieces[self.current_team_index].push(piece);
    }

    pub fn get_unused_piece(&mut self) -> Option<Entity> {
        self.unused_pieces[self.current_team_index].pop()
    }
    pub fn discard_unused_piece(&mut self) -> Option<Entity> {
        self.get_unused_piece()
    }

    pub fn num_unused_pieces(&self) -> usize {
        self.unused_pieces[self.current_team_index].len()
    }

    pub fn place_piece(&mut self, piece: Entity, x: u8, y: u8){
        self.placed_pieces[x as usize][y as usize] = Some(piece);
    }

    pub fn place_piece_at(&mut self, piece: Entity, pos: &BoardPosition){
        self.place_piece(piece, pos.coords.x, pos.coords.y);
    }

    pub fn remove_piece(&mut self, x: u8, y: u8){
        self.placed_pieces[x as usize][y as usize] = None;
    }
    pub fn remove_piece_at(&mut self, pos: &BoardPosition){
        self.remove_piece(pos.coords.x, pos.coords.y);
    }

    pub fn get_piece(&self, x: u8, y: u8) -> Option<Entity> {
        self.placed_pieces[x as usize][y as usize]
    }

    pub fn get_piece_at(&self, pos: &BoardPosition) -> Option<Entity> {
        self.get_piece(pos.coords.x, pos.coords.y)
    }

    pub fn move_piece(&mut self, piece: Entity, from_x: u8, from_y: u8, to_x: u8, to_y: u8){
        self.remove_piece(from_x, from_y);
        self.place_piece(piece, to_x, to_y);
    }
    pub fn move_piece_at(&mut self, piece: Entity, from_pos: &BoardPosition, to_pos: &BoardPosition){
        self.remove_piece_at(from_pos);
        self.place_piece_at(piece, to_pos);
    }

    pub fn get_cell_safely(&self, x: i16, y: i16) -> Option<Entity> {
        if x >= 0 && y >=0 && x < self.w as i16 && y < self.h as i16{
            Some(self.get_cell(x as u8,y as u8))
        } else {
            None
        }
    }

    pub fn get_cell(&self, x: u8, y: u8) -> Entity {
        self.cells[x as usize][y as usize]
    }

    pub fn current_team(&self) -> Team {
        self.teams[self.current_team_index]
    }

    pub fn is_current_team(&self, team_id: usize) -> bool {
        self.current_team_index == team_id
    }

    pub fn get_team(&self, team_id: usize) -> Team {
        self.teams[team_id]
    }

    pub fn num_teams(&self) -> usize {
        self.teams.len()
    }

    pub fn mark_team_as_lost(&mut self, team_id: usize) {
        self.teams[team_id].lost = true;
    }

    pub fn next_team(&mut self) -> Option<Team> {
        let initial_team_index = self.current_team_index;
        println!("From team {} to next team.", self.current_team_index);

        loop {
            self.current_team_index += 1;
            if self.current_team_index >= self.teams.len() {
                self.current_team_index = 0;
            }

            if self.current_team_index == initial_team_index {
                println!("No next team. Current team {}", self.current_team_index);
                return None // All (other) teams lost
            } else if !self.current_team().lost {
                println!("Next team is {}", self.current_team_index);
                return Some(self.current_team());
            }
        }
    }



    pub fn set_event(&mut self, event: BoardEvent) {
        self.event = Some(event);
    }
    pub fn poll_event(&mut self) -> Option<BoardEvent> {

        let event = self.event.take();
        if event.is_some() {
            println!("Handling event {:?}", event);
        }
        event
    }

    pub fn match_pattern(&self, pattern: &Pattern, start_x: u8, start_y: u8) -> Option<Vec<Entity>> {
        let mut matched_entities = Vec::new();
        for (pattern_y, line) in pattern.components.iter().enumerate() {
            for (pattern_x, p) in line.iter().enumerate() {
                let board_x = start_x + pattern_x as u8;
                let board_y = start_y + pattern_y as u8;

                if let Some(piece) = self.get_piece(board_x, board_y) {
                    if p == &PatternComponent::Free {
                        return None;
                    }
                    else if p == &PatternComponent::OwnPiece {
                        matched_entities.push(piece);
                    }
                } else if p == &PatternComponent::OwnPiece {
                    return None;
                }
            }
        }

        Option::Some(matched_entities)
    }
}

#[derive(Debug, PartialEq)]
pub enum PatternComponent {
    OwnPiece,
    Free,
    Any,
}

pub struct Pattern {
    pub components: Vec<Vec<PatternComponent>>,
    pub turn_into: PieceKind,
    pub new_piece_relative_position: Point2<u8>,
}

impl Pattern {
    pub fn all_patterns() -> [Pattern;6] {
        [
            Pattern {
                components: vec![
                    vec![PatternComponent::Any,     PatternComponent::Any, PatternComponent::OwnPiece,  PatternComponent::Any,      PatternComponent::Any],
                    vec![PatternComponent::Any,     PatternComponent::OwnPiece, PatternComponent::Free, PatternComponent::OwnPiece, PatternComponent::Any],
                    vec![PatternComponent::OwnPiece,PatternComponent::Free,     PatternComponent::Free, PatternComponent::Free,     PatternComponent::OwnPiece],
                    vec![PatternComponent::Any,     PatternComponent::OwnPiece, PatternComponent::Free, PatternComponent::OwnPiece, PatternComponent::Any],
                    vec![PatternComponent::Any,     PatternComponent::Any, PatternComponent::OwnPiece,  PatternComponent::Any,      PatternComponent::Any],
                ],
                turn_into: PieceKind::Queen,
                new_piece_relative_position: Point2::new(2, 2)
            },
            Pattern {
                components: vec![
                    vec![PatternComponent::Any,     PatternComponent::OwnPiece, PatternComponent::Any],
                    vec![PatternComponent::OwnPiece,PatternComponent::OwnPiece, PatternComponent::OwnPiece],
                    vec![PatternComponent::Any,     PatternComponent::OwnPiece, PatternComponent::Any],
                ],
                turn_into: PieceKind::Cross,
                new_piece_relative_position: Point2::new(1, 1)
            },
            Pattern {
                components: vec![
                    vec![PatternComponent::Free, PatternComponent::Free, PatternComponent::Free],
                    vec![PatternComponent::OwnPiece, PatternComponent::OwnPiece, PatternComponent::OwnPiece],
                    vec![PatternComponent::Free, PatternComponent::Free, PatternComponent::Free],
                ],
                turn_into: PieceKind::HorizontalBar,
                new_piece_relative_position: Point2::new(1, 1)
            },
            Pattern {
                components: vec![
                    vec![PatternComponent::Free,PatternComponent::OwnPiece,PatternComponent::Free],
                    vec![PatternComponent::Free,PatternComponent::OwnPiece,PatternComponent::Free],
                    vec![PatternComponent::Free,PatternComponent::OwnPiece,PatternComponent::Free],
                ],
                turn_into: PieceKind::VerticalBar,
                new_piece_relative_position: Point2::new(1, 1)
            },
            Pattern {
                components: vec![
                    vec![PatternComponent::OwnPiece,PatternComponent::Any,PatternComponent::OwnPiece],
                    vec![PatternComponent::Any,PatternComponent::OwnPiece,PatternComponent::Any],
                    vec![PatternComponent::OwnPiece,PatternComponent::Any,PatternComponent::OwnPiece],
                ],
                turn_into: PieceKind::Sniper,
                new_piece_relative_position: Point2::new(1, 1)
            },
            Pattern {
                components: vec![
                    vec![PatternComponent::Any,PatternComponent::OwnPiece,PatternComponent::Any],
                    vec![PatternComponent::OwnPiece,PatternComponent::Free,PatternComponent::OwnPiece],
                    vec![PatternComponent::Any,PatternComponent::OwnPiece,PatternComponent::Any],
                ],
                turn_into: PieceKind::Castle,
                new_piece_relative_position: Point2::new(1, 1)
            },


        ]
    }
}