use crate::{Color, Piece, Square};
use std::fmt;

// ........ ........ ........ .####### : to
// ........ ........ ........ #....... : promotion flag
// ........ ........ .####### ........ : from (0 if drop move)
// ........ ........ #....... ........ : drop flag
// ........ ...##### ........ ........ : moved piece or dropped piece type
// ...##### ........ ........ ........ : captured piece (if moved)
#[derive(Clone, Copy)]
pub struct Move(u32);

impl Move {
    const SQUARE_MASK: u32 = 0x0000_007f;
    const FROM_MASK: u32 = 0x0000_7f00;
    const FROM_SHIFT: u32 = 8;
    const PROMOTION_FLAG: u32 = 1 << 7;
    const DROP_FLAG: u32 = 1 << 15;
    const PIECE_MASK: u32 = 0x001f_0000;
    const PIECE_SHIFT: u32 = 16;
    const CAPTURED_SHIFT: u32 = 24;

    pub fn new_normal(
        from: Square,
        to: Square,
        is_promotion: bool,
        piece: Piece,
        captured: Piece,
    ) -> Self {
        Move(
            to.index() as u32
                | ((from.0 as u32) << Move::FROM_SHIFT)
                | if is_promotion {
                    Move::PROMOTION_FLAG
                } else {
                    0
                }
                | (piece.0 as u32) << Move::PIECE_SHIFT
                | (captured.0 as u32) << Move::CAPTURED_SHIFT,
        )
    }
    pub fn new_drop(to: Square, piece: Piece) -> Self {
        Move(to.index() as u32 | Move::DROP_FLAG | (piece.0 as u32) << Move::PIECE_SHIFT)
    }
    pub fn from(&self) -> Option<Square> {
        if self.is_drop() {
            None
        } else {
            Some(Square(
                ((self.0 & Move::FROM_MASK) >> Move::FROM_SHIFT) as i8,
            ))
        }
    }
    pub fn to(&self) -> Square {
        Square((self.0 & Move::SQUARE_MASK) as i8)
    }
    pub fn is_promotion(&self) -> bool {
        (self.0 & Move::PROMOTION_FLAG) != 0
    }
    pub fn is_drop(&self) -> bool {
        (self.0 & Move::DROP_FLAG) != 0
    }
    pub fn piece(&self) -> Piece {
        Piece(((self.0 & Move::PIECE_MASK) >> Move::PIECE_SHIFT) as u8)
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.piece().color().expect("failed to get piece color") {
                Color::Black => '+',
                Color::White => '-',
            },
        )?;
        if let Some(from) = self.from() {
            write!(f, "{}{}", from.file(), from.rank())?;
        } else {
            write!(f, "00")?;
        }
        write!(f, "{}{}", self.to().file(), self.to().rank())?;
        write!(
            f,
            "{}",
            self.piece().piece_type().expect("failed to get piece type")
        )?;
        Ok(())
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
