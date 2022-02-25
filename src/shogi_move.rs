use std::fmt;

use crate::{Piece, Square};

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

    pub fn new(from: Square, to: Square, piece: Piece, promote: bool) -> Self {
        Move(
            (piece.0 as u32) << Move::PIECE_SHIFT
                | (from.0 as u32) << Move::FROM_SHIFT
                | (to.0 as u32),
        )
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Move")
            .field(
                "from",
                &Square(((self.0 >> Move::FROM_SHIFT) & Move::SQUARE_MASK) as i8),
            )
            .field("to", &Square((self.0 & Move::SQUARE_MASK) as i8))
            .field(
                "piece",
                &Piece(((self.0 >> Move::PIECE_SHIFT) & Move::PIECE_MASK) as u8),
            )
            .finish()
    }
}
