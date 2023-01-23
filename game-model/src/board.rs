use std::fmt::Display;

use crate::{
    piece::*,
    Point2,
};
use colored::Colorize;
use nanoserde::{DeJson, SerJson};

#[derive(Clone, PartialEq, Eq, Debug, DeJson, SerJson)]
pub struct Cell {
    pub point: Point2,
    pub piece: Option<Piece>,
    pub effects: Vec<EffectKind>,
}

#[derive(Clone, PartialEq, Debug, Eq, DeJson, SerJson)]
pub struct Board {
    pub(crate) cells: Vec<Vec<Cell>>,
    pub w: u8,
    pub h: u8,
}

impl Board {
    pub fn new(width: u8, height: u8) -> Board {
        let mut cells = vec![];

        for x in 0..width {
            let mut column = vec![];
            for y in 0..height {
                column.push(Cell {
                    point: Point2::new(x, y),
                    piece: Option::None,
                    effects: vec![],
                });
            }
            cells.push(column);
        }

        Board {
            cells,
            w: width,
            h: height,
        }
    }

    pub fn for_each_cell_mut<F>(&mut self, mut closure: F)
    where
        F: FnMut(&mut Cell) -> (),
    {
        for row in self.cells.iter_mut() {
            for cell in row {
                closure(cell);
            }
        }
    }

    pub fn for_each_placed_piece_mut<F>(&mut self, mut closure: F)
    where
        F: FnMut(Point2, &mut Piece) -> (),
    {
        self.for_each_cell_mut(|cell| {
            if let Some(piece) = cell.piece.as_mut() {
                closure(cell.point, piece);
            }
        });
    }
    pub fn for_each_cell<F>(&self, mut closure: F)
    where
        F: FnMut(&Cell) -> (),
    {
        for row in self.cells.iter() {
            for cell in row {
                closure(cell);
            }
        }
    }

    pub fn for_each_placed_piece<F>(&self, mut closure: F)
    where
        F: FnMut(Point2, &Piece) -> (),
    {
        self.for_each_cell(|cell| {
            if let Some(piece) = cell.piece.as_ref() {
                closure(cell.point, piece);
            }
        });
    }

    pub fn placed_pieces(&self, team: usize) -> Vec<Piece> {
        let mut pieces = vec![];

        self.for_each_placed_piece(|_point, piece| {
            if piece.team_id == team {
                pieces.push(*piece);
            }
        });

        pieces
    }

    pub fn has_effect_at(&self, effect: &EffectKind, pos: &Point2) -> bool {
        self.get_cell(pos).effects.contains(effect)
    }

    pub fn get_piece_at(&self, pos: &Point2) -> Option<&Piece> {
        if !self.has_cell(pos) {
            return Option::None;
        }
        self.cells[pos.x as usize][pos.y as usize].piece.as_ref()
    }

    pub fn get_piece_mut(&mut self, x: u8, y: u8) -> Option<&mut Piece> {
        self.cells[x as usize][y as usize].piece.as_mut()
    }

    pub fn get_piece_mut_at(&mut self, pos: &Point2) -> Option<&mut Piece> {
        self.get_piece_mut(pos.x, pos.y)
    }

    pub fn has_cell(&self, point: &Point2) -> bool {
        point.x < self.w && point.y < self.h
    }

    pub fn place_piece_at(&mut self, piece: Piece, pos: &Point2) {
        let target_cell = self.get_cell_mut(pos);

        assert!(
            target_cell.piece.is_none(),
            "Can't place on top of another piece at {:?}",
            pos
        );

        target_cell.piece = Some(piece);
    }

    pub fn add_effect(&mut self, kind: EffectKind, pos: &Point2) {
        self.get_cell_mut(pos).effects.push(kind);
    }

    pub fn remove_effect(&mut self, kind: &EffectKind, pos: &Point2) {
        let effects = &mut self.get_cell_mut(pos).effects;
        let index = effects.iter().position(|e| e == kind).expect(
            format!(
                "Can't remove effect {:?} at {:?} because it doesn't exist",
                kind, pos
            )
            .as_str(),
        );
        effects.swap_remove(index);
    }

    fn get_cell_mut(&mut self, pos: &Point2) -> &mut Cell {
        &mut self.cells[pos.x as usize][pos.y as usize]
    }
    fn get_cell(&self, pos: &Point2) -> &Cell {
        &self.cells[pos.x as usize][pos.y as usize]
    }

    pub fn remove_piece_at(&mut self, pos: &Point2) {
        self.get_cell_mut(pos)
            .piece
            .take()
            .expect(format!("Cannot remove: There is no piece on {:?}", pos).as_str());
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.h as usize {
            for x in 0..self.w as usize {
                write!(f, "{}", self.cells[x][y])?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cell = if let Some(piece) = self.piece {
            format!("{}", piece)
        } else {
            ".".to_string()
        };

        let cell = if !self.effects.is_empty() {
            cell.on_bright_magenta()
        } else {
            cell.normal()
        };

        write!(f, "{}", cell)
    }
}
