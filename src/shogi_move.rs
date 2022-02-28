use crate::{Piece, Square};
use std::fmt;

pub enum MoveType {
    Normal,
    Drop,
}

// xxxxxxxx xxxxxxxx xxxxxxxx x1111111  to
// xxxxxxxx xxxxxxxx xxxxxxxx 1xxxxxxx  promote flag
// xxxxxxxx xxxxxxxx x1111111 xxxxxxxx  from (or 0 if drop)
// xxxxxxxx xxxxxxxx 1xxxxxxx xxxxxxxx  drop flag
// xxxxxxxx xxx11111 xxxxxxxx xxxxxxxx  moved piece (If this move is promotion, moved piece is unpromoted piece)
#[derive(Clone, Copy)]
pub struct Move(u32);

impl Move {
    const SQUARE_MASK: u32 = 0x0000_007f;
    const PIECE_MASK: u32 = 0x0000_001f;
    const FROM_SHIFT: u32 = 8;
    const PIECE_SHIFT: u32 = 16;
    const PROMOTE_FLAG: u32 = 1 << 7;
    const DROP_FLAG: u32 = 1 << 15;

    pub fn new(from: Square, to: Square, piece: Piece, promote: bool) -> Self {
        Move(
            (piece.0 as u32) << Move::PIECE_SHIFT
                | if promote { Move::PROMOTE_FLAG } else { 0 }
                | (from.0 as u32) << Move::FROM_SHIFT
                | (to.0 as u32),
        )
    }
    pub fn from(&self) -> Square {
        Square(((self.0 >> Move::FROM_SHIFT) & Move::SQUARE_MASK) as i8)
    }
    pub fn to(&self) -> Square {
        Square((self.0 & Move::SQUARE_MASK) as i8)
    }
    pub fn move_type(&self) -> MoveType {
        if self.0 & Move::DROP_FLAG == 0 {
            MoveType::Normal
        } else {
            MoveType::Drop
        }
    }
    pub fn is_promotion(&self) -> bool {
        self.0 & Move::PROMOTE_FLAG != 0
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Move")
            .field("from", &self.from())
            .field("to", &self.to())
            .field(
                "piece",
                &Piece(((self.0 >> Move::PIECE_SHIFT) & Move::PIECE_MASK) as u8),
            )
            .field("promotion", &self.is_promotion())
            .finish()
    }
}
