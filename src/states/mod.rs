pub mod pieceplacement;
pub mod load;
pub mod piecemovement;
pub mod next_turn;
pub mod game_over;
pub mod target_for_power;

pub use self::pieceplacement::PiecePlacementState;
pub use self::piecemovement::PieceMovementState;
pub use self::load::LoadingState;
