use crate::{Piece, Square};
use std::fmt;

/// Represents a state of the game.
#[derive(Debug)]
pub struct Position {
    board: [Piece; Square::NUM],
}

impl Position {
    pub fn new(board: [Piece; Square::NUM]) -> Position {
        Position { board }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            #[rustfmt::skip]
            board: [
                Piece::WKY, Piece::WKE, Piece::WGI, Piece::WKI, Piece::WOU, Piece::WKI, Piece::WGI, Piece::WKE, Piece::WKY,
                Piece::EMP, Piece::WHI, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::WKA, Piece::EMP,
                Piece::WFU, Piece::WFU, Piece::WFU, Piece::WFU, Piece::WFU, Piece::WFU, Piece::WFU, Piece::WFU, Piece::WFU,
                Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP,
                Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP,
                Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP,
                Piece::BFU, Piece::BFU, Piece::BFU, Piece::BFU, Piece::BFU, Piece::BFU, Piece::BFU, Piece::BFU, Piece::BFU,
                Piece::EMP, Piece::BKA, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::BHI, Piece::EMP,
                Piece::BKY, Piece::BKE, Piece::BGI, Piece::BKI, Piece::BOU, Piece::BKI, Piece::BGI, Piece::BKE, Piece::BKY,
            ],
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in 0..9 {
            write!(f, "P{}", rank + 1)?;
            for file in 0..9 {
                write!(f, "{}", self.board[rank * 9 + file])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
