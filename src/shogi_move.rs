use shogi_core::{Color, Piece, PieceKind, Square};
use std::fmt;

pub enum MoveType {
    Normal {
        from: Square,
        to: Square,
        is_promotion: bool,
        piece: Piece,
    },
    Drop {
        to: Square,
        piece: Piece,
    },
}

// ........ ........ ........ .####### : to
// ........ ........ ........ #....... : promotion flag
// ........ ........ .####### ........ : from (0 if drop move)
// ........ ........ #....... ........ : drop flag
// ........ ...##### ........ ........ : moved or dropped piece
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move(u32);

impl Move {
    const TO_MASK: u32 = 0x0000_007f;
    const FROM_SHIFT: u32 = 8;
    const FROM_MASK: u32 = 0x0000_7f00;
    const PROMOTION_FLAG: u32 = 1 << 7;
    const DROP_FLAG: u32 = 1 << 15;
    const PIECE_SHIFT: u32 = 16;
    const PIECE_MASK: u32 = 0x001f_0000;

    pub fn new_normal(from: Square, to: Square, is_promotion: bool, piece: Piece) -> Self {
        Move(
            (to.index() as u32 - 1)
                | ((from.index() as u32 - 1) << Move::FROM_SHIFT)
                | if is_promotion {
                    Move::PROMOTION_FLAG
                } else {
                    0
                }
                | (piece.as_u8() as u32) << Move::PIECE_SHIFT,
        )
    }
    pub fn new_drop(to: Square, piece: Piece) -> Self {
        Move(
            (to.index() as u32 - 1) | Move::DROP_FLAG | (piece.as_u8() as u32) << Move::PIECE_SHIFT,
        )
    }
    pub fn from(&self) -> Option<Square> {
        if self.is_drop() {
            None
        } else {
            Square::from_u8(((self.0 & Move::FROM_MASK) >> Move::FROM_SHIFT) as u8 + 1)
        }
    }
    pub fn move_type(&self) -> MoveType {
        if let Some(from) = self.from() {
            MoveType::Normal {
                from,
                to: self.to(),
                is_promotion: self.is_promotion(),
                piece: self.piece(),
            }
        } else {
            MoveType::Drop {
                to: self.to(),
                piece: self.piece(),
            }
        }
    }
    pub fn to(&self) -> Square {
        unsafe { Square::from_u8_unchecked((self.0 & Move::TO_MASK) as u8 + 1) }
    }
    pub fn is_promotion(&self) -> bool {
        (self.0 & Move::PROMOTION_FLAG) != 0
    }
    pub fn is_drop(&self) -> bool {
        (self.0 & Move::DROP_FLAG) != 0
    }
    pub fn piece(&self) -> Piece {
        // Piece(((self.0 & Move::PIECE_MASK) >> Move::PIECE_SHIFT) as u8)
        let data = ((self.0 & Move::PIECE_MASK) >> Move::PIECE_SHIFT) as u8;
        let disc = data & 15;
        Piece::new(
            unsafe { PieceKind::from_u8_unchecked(disc) },
            if data >= 16 {
                Color::White
            } else {
                Color::Black
            },
        )
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Move")
            .field("from", &self.from())
            .field("to", &self.to())
            .field("promotion", &self.is_promotion())
            .field("piece", &self.piece())
            .finish()
    }
}
