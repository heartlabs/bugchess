use crate::board::Board;

pub struct Game {
    pub board: Board,
    pub teams: Vec<Team>,
    pub current_team_index: usize,
}

#[derive(Clone, Copy)]
pub struct Team {
    pub id: usize,
//    pub color: Color,
    pub name: &'static str,
    pub lost: bool,
    pub unused_pieces: u8,
}

impl Game {
    pub fn new(teams: Vec<Team>, board_width: u8, board_height: u8) -> Self {
        Game {
            board: Board::new(board_width, board_height),
            teams,
            current_team_index: 0,
        }
    }

    pub fn num_unused_pieces_of(&self, team_id: usize) -> u8 {
        self.get_team(team_id).unused_pieces
    }

    pub fn current_team(&self) -> Team {
        self.teams[self.current_team_index]
    }

    pub fn is_current_team(&self, team_id: usize) -> bool {
        self.current_team_index == team_id
    }

    pub fn get_team(&self, team_id: usize) -> &Team {
        &self.teams[team_id]
    }

    pub fn num_teams(&self) -> usize {
        self.teams.len()
    }

    pub fn mark_team_as_lost(&mut self, team_id: usize) {
        println!("Team {} lost.", team_id);
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
                return None; // All (other) teams lost
            } else if !self.current_team().lost {
                println!("Next team is {}", self.current_team_index);
                return Some(self.current_team());
            }
        }
    }

    pub fn add_unused_piece_for(&mut self, team_id: usize) {
        self.teams[team_id].unused_pieces += 1;
    }

    pub fn remove_unused_piece(&mut self, team_id: usize) -> bool {
        if self.teams[team_id].unused_pieces <= 0 {
            false
        } else {
            self.teams[team_id].unused_pieces -= 1;
            true
        }
    }

    pub fn unused_piece_available(&self) -> bool {
        self.current_team().unused_pieces > 0
    }
}
