use crate::ranges::*;
use colored::Colorize;
use nanoserde::{DeBin, SerBin};
use std::fmt::{Debug, Display};

#[derive(Debug, Copy, Clone, PartialEq, Eq, SerBin, DeBin)]
pub enum EffectKind {
    Protection,
}

impl Move {
    pub fn new(direction: Direction, steps: u8) -> Move {
        Move {
            range: Range {
                direction,
                context: RangeContext::Moving,
                steps,
                jumps: false,
                include_self: false,
            },
        }
    }
    pub fn new_unlimited(direction: Direction) -> Move {
        Move {
            range: Range {
                direction,
                context: RangeContext::Moving,
                steps: 255,
                jumps: false,
                include_self: false,
            },
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, SerBin, DeBin)]
pub struct ActivatablePower {
    pub kind: Power,
    pub range: Range,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, SerBin, DeBin)]
pub struct Move {
    pub range: Range,
}

#[derive(Debug)]
pub struct TurnInto {
    pub kind: PieceKind,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, SerBin, DeBin)]
pub enum Power {
    Blast,
    TargetedShoot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, SerBin, DeBin)]
pub enum PieceKind {
    Simple,
    HorizontalBar,
    VerticalBar,
    Cross,
    Queen,
    Castle,
    Sniper,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, SerBin, DeBin)]
pub struct Effect {
    pub kind: EffectKind,
    pub range: Range,
}

impl Piece {
    pub fn simple() -> Piece {
        Self::new(0, PieceKind::Simple)
    }

    pub fn new(team_id: usize, turn_into: PieceKind) -> Piece {
        let mut piece = Piece {
            piece_kind: PieceKind::Simple,
            attack: true,
            pierce: true,
            shield: false,
            movement: None,
            activatable: None,
            effect: None,
            exhaustion: Exhaustion::new_exhausted(ExhaustionStrategy::Either),
            team_id,
        };

        Piece::init_piece(&mut piece, turn_into);

        piece
    }

    fn init_piece(piece: &mut Piece, turn_into: PieceKind) {
        piece.piece_kind = turn_into;
        match turn_into {
            PieceKind::HorizontalBar => {
                piece.movement = Some(Move::new_unlimited(Direction::Horizontal));
                piece.activatable = Some(ActivatablePower {
                    range: Range::new_unlimited(Direction::Horizontal, RangeContext::Moving),
                    kind: Power::Blast,
                });
            }
            PieceKind::VerticalBar => {
                piece.movement = Some(Move::new_unlimited(Direction::Vertical));
                piece.activatable = Some(ActivatablePower {
                    range: Range::new_unlimited(Direction::Vertical, RangeContext::Moving),
                    kind: Power::Blast,
                });
            }
            PieceKind::Cross => {
                piece.movement = Some(Move::new_unlimited(Direction::Straight));
                piece.shield = true;
            }
            PieceKind::Simple => {
                piece.movement = Some(Move::new(Direction::Star, 1));
                piece.pierce = false;
            }
            PieceKind::Queen => {
                piece.movement = Some(Move::new_unlimited(Direction::Star));
                piece.exhaustion = Exhaustion::new_exhausted(ExhaustionStrategy::Both);
                piece.activatable = Some(ActivatablePower {
                    range: Range {
                        direction: Direction::Star,
                        context: RangeContext::Special,
                        steps: 1,
                        jumps: false,
                        include_self: false,
                    },
                    kind: Power::Blast,
                });
            }
            PieceKind::Castle => {
                piece.shield = true;
                piece.effect = Some(Effect {
                    kind: EffectKind::Protection,
                    range: Range {
                        direction: Direction::Star,
                        context: RangeContext::Area,
                        steps: 1,
                        jumps: true,
                        include_self: true,
                    },
                });
            }
            PieceKind::Sniper => {
                piece.activatable = Some(ActivatablePower {
                    range: Range {
                        direction: Direction::Anywhere,
                        context: RangeContext::Special,
                        steps: 0,
                        jumps: false,
                        include_self: false,
                    },
                    kind: Power::TargetedShoot,
                });
            }
        }
    }

    pub fn can_move(&self) -> bool {
        self.exhaustion.can_move() && self.movement.is_some()
    }

    pub fn can_use_special(&self) -> bool {
        self.exhaustion.can_attack() && self.activatable.is_some()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, SerBin, DeBin)]
pub struct Piece {
    pub piece_kind: PieceKind,
    pub attack: bool,
    pub pierce: bool,
    pub shield: bool,
    pub movement: Option<Move>,
    pub activatable: Option<ActivatablePower>,
    pub effect: Option<Effect>,
    pub exhaustion: Exhaustion,
    pub team_id: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, SerBin, DeBin)]
pub enum ExhaustionStrategy {
    Either,
    Both,
    Move,
    Special,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, SerBin, DeBin)]
pub struct Exhaustion {
    moved: bool,
    used_special: bool,
    strategy: ExhaustionStrategy,
}

impl Exhaustion {
    pub(crate) fn new_exhausted(strategy: ExhaustionStrategy) -> Exhaustion {
        Exhaustion {
            moved: true,
            used_special: true,
            strategy,
        }
    }

    pub fn can_move(&self) -> bool {
        match &self.strategy {
            ExhaustionStrategy::Special => false,
            ExhaustionStrategy::Move | ExhaustionStrategy::Both => !self.moved,
            ExhaustionStrategy::Either => !self.moved && !self.used_special,
        }
    }

    pub fn can_attack(&self) -> bool {
        match &self.strategy {
            ExhaustionStrategy::Move => false,
            ExhaustionStrategy::Special | ExhaustionStrategy::Both => !self.used_special,
            ExhaustionStrategy::Either => !self.moved && !self.used_special,
        }
    }

    pub fn is_done(&self) -> bool {
        !self.can_move() && !self.can_attack()
    }

    pub fn on_move(&mut self) {
        self.moved = true;
    }

    pub fn on_attack(&mut self) {
        self.used_special = true;
    }

    pub fn reset(&mut self) {
        self.moved = false;
        self.used_special = false;
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self.piece_kind {
            PieceKind::Simple => "o",
            PieceKind::HorizontalBar => "-",
            PieceKind::VerticalBar => "|",
            PieceKind::Cross => "+",
            PieceKind::Queen => "*",
            PieceKind::Castle => "W",
            PieceKind::Sniper => "x",
        };

        let kind = match self.team_id {
            0 => kind.red(),
            1 => kind.yellow(),
            _ => kind.normal(),
        };

        write!(f, "{}", kind)
    }
}
