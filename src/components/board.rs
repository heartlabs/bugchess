use amethyst::core::math::{Point2};
use amethyst::ecs::{Component, DenseVecStorage, Entity, WriteStorage};
use amethyst::renderer::palette::Srgba;
use ncollide3d::narrow_phase::EventPool;

pub const BOARD_WIDTH: usize = 5;
pub const BOARD_HEIGHT: usize = 5;


#[derive(Debug, Copy, Clone)]
pub enum BoardEvent {
    Cell {
        x: usize,
        y: usize
    },
    Next,
    None,
}

impl Default for BoardEvent {
    fn default() -> BoardEvent {
        BoardEvent::None
    }
}

#[derive(Component, Clone, Copy)]
#[storage(DenseVecStorage)]
pub struct Team {
    pub id: usize,
    pub color: Srgba,
    pub name: &'static str
}

#[derive(Component, Clone, Copy)]
#[storage(DenseVecStorage)]
pub struct Piece {
    //pub team: &'static Team
}

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct Cell {
    pub coords: Point2<usize>,
    pub piece: Option<&'static mut Piece>,
}

impl Cell {
    pub const width: usize = 64;
    pub const height: usize = 64;
}

pub struct Board {
    cells: Vec<Vec<Entity>>,
    pieces: Vec<Vec<Option<Entity>>>,
    teams: Vec<Team>,
    current_team_index: usize,
    pub w: usize,
    pub h: usize,
    event: Option<BoardEvent>,
}

impl Board {
    pub fn new(cells: Vec<Vec<Entity>>, teams: Vec<Team>) -> Board {
        let pieces = (0..BOARD_WIDTH)
            .map(|i| {
                (0..BOARD_HEIGHT)
                    .map(|j| {
                        Option::None
                    })
                    .collect()
            })
            .collect();

        Board {
            cells,
            pieces,
            teams,
            current_team_index: 0,
            w: BOARD_WIDTH,
            h: BOARD_HEIGHT,
            event: Option::None,
        }
    }

    pub fn placePiece(&mut self, piece: Entity, x: usize, y: usize){
        self.pieces[x][y] = Some(piece);
    }

    pub fn remove_piece(&mut self, x: usize, y: usize){
        self.pieces[x][y] = None;
    }

    pub fn get_piece(&self, x: usize, y: usize) -> Option<Entity> {
        self.pieces[x][y]
    }

    pub fn move_piece(&mut self, piece: Entity, from_x: usize, from_y: usize, to_x: usize, to_y: usize){
        self.remove_piece(from_x, from_y);
        self.placePiece(piece, to_x, to_y);
    }

    pub fn get_cell(&self, x: usize, y: usize) -> Entity {
        self.cells[x][y]
    }

    pub fn current_team(&mut self) -> Team {
        self.teams[self.current_team_index]
    }

    pub fn next_team(&mut self) -> Team {
        self.current_team_index += 1;
        if (self.current_team_index >= self.teams.len()){
            self.current_team_index = 0;
        }

        self.current_team()
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

}
