use crate::board::{Board};
use crate::piece::PieceKind;
use crate::Point2;

#[derive(Debug, PartialEq, Eq)]
pub enum PatternComponent {
    OwnPiece,
    Free,
    Any,
}

pub struct Pattern {
    pub components: Vec<Vec<PatternComponent>>,
    pub turn_into: PieceKind,
    pub new_piece_relative_position: Point2,
}

impl Pattern {
    pub fn all_patterns() -> [Pattern; 6] {
        // TODO make static var instead
        [
            Pattern {
                components: vec![
                    vec![
                        PatternComponent::Any,
                        PatternComponent::Any,
                        PatternComponent::OwnPiece,
                        PatternComponent::Any,
                        PatternComponent::Any,
                    ],
                    vec![
                        PatternComponent::Any,
                        PatternComponent::OwnPiece,
                        PatternComponent::Free,
                        PatternComponent::OwnPiece,
                        PatternComponent::Any,
                    ],
                    vec![
                        PatternComponent::OwnPiece,
                        PatternComponent::Free,
                        PatternComponent::Free,
                        PatternComponent::Free,
                        PatternComponent::OwnPiece,
                    ],
                    vec![
                        PatternComponent::Any,
                        PatternComponent::OwnPiece,
                        PatternComponent::Free,
                        PatternComponent::OwnPiece,
                        PatternComponent::Any,
                    ],
                    vec![
                        PatternComponent::Any,
                        PatternComponent::Any,
                        PatternComponent::OwnPiece,
                        PatternComponent::Any,
                        PatternComponent::Any,
                    ],
                ],
                turn_into: PieceKind::Queen,
                new_piece_relative_position: Point2::new(2, 2),
            },
            Pattern {
                components: vec![
                    vec![
                        PatternComponent::Any,
                        PatternComponent::OwnPiece,
                        PatternComponent::Any,
                    ],
                    vec![
                        PatternComponent::OwnPiece,
                        PatternComponent::OwnPiece,
                        PatternComponent::OwnPiece,
                    ],
                    vec![
                        PatternComponent::Any,
                        PatternComponent::OwnPiece,
                        PatternComponent::Any,
                    ],
                ],
                turn_into: PieceKind::Cross,
                new_piece_relative_position: Point2::new(1, 1),
            },
            Pattern {
                components: vec![
                    vec![
                        PatternComponent::Free,
                        PatternComponent::Free,
                        PatternComponent::Free,
                    ],
                    vec![
                        PatternComponent::OwnPiece,
                        PatternComponent::OwnPiece,
                        PatternComponent::OwnPiece,
                    ],
                    vec![
                        PatternComponent::Free,
                        PatternComponent::Free,
                        PatternComponent::Free,
                    ],
                ],
                turn_into: PieceKind::HorizontalBar,
                new_piece_relative_position: Point2::new(1, 1),
            },
            Pattern {
                components: vec![
                    vec![
                        PatternComponent::Free,
                        PatternComponent::OwnPiece,
                        PatternComponent::Free,
                    ],
                    vec![
                        PatternComponent::Free,
                        PatternComponent::OwnPiece,
                        PatternComponent::Free,
                    ],
                    vec![
                        PatternComponent::Free,
                        PatternComponent::OwnPiece,
                        PatternComponent::Free,
                    ],
                ],
                turn_into: PieceKind::VerticalBar,
                new_piece_relative_position: Point2::new(1, 1),
            },
            Pattern {
                components: vec![
                    vec![
                        PatternComponent::OwnPiece,
                        PatternComponent::Any,
                        PatternComponent::OwnPiece,
                    ],
                    vec![
                        PatternComponent::Any,
                        PatternComponent::OwnPiece,
                        PatternComponent::Any,
                    ],
                    vec![
                        PatternComponent::OwnPiece,
                        PatternComponent::Any,
                        PatternComponent::OwnPiece,
                    ],
                ],
                turn_into: PieceKind::Sniper,
                new_piece_relative_position: Point2::new(1, 1),
            },
            Pattern {
                components: vec![
                    vec![
                        PatternComponent::Any,
                        PatternComponent::OwnPiece,
                        PatternComponent::Any,
                    ],
                    vec![
                        PatternComponent::OwnPiece,
                        PatternComponent::Free,
                        PatternComponent::OwnPiece,
                    ],
                    vec![
                        PatternComponent::Any,
                        PatternComponent::OwnPiece,
                        PatternComponent::Any,
                    ],
                ],
                turn_into: PieceKind::Castle,
                new_piece_relative_position: Point2::new(1, 1),
            },
        ]
    }

    pub fn match_board(
        &self,
        board: &Board,
        start_x: u8,
        start_y: u8,
    ) -> Option<Vec<Point2>> {
        let mut matched_entities = Vec::new();
        for (pattern_y, line) in self.components.iter().enumerate() {
            for (pattern_x, p) in line.iter().enumerate() {
                let board_x = start_x + pattern_x as u8;
                let board_y = start_y + pattern_y as u8;

                let board_point = Point2::new(board_x, board_y);

                if let Some(_piece) = board.get_piece_at(&board_point) {
                    if p == &PatternComponent::Free {
                        return None;
                    } else if p == &PatternComponent::OwnPiece {
                        matched_entities.push(board_point);
                    }
                } else if p == &PatternComponent::OwnPiece {
                    return None;
                }
            }
        }

        Option::Some(matched_entities)
    }
}
