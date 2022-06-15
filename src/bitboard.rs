use shogi_core::Square;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, Shr};

pub(crate) trait Occupied {
    fn sliding_left_down(&self, mask: &Self) -> Self;
    fn sliding_right_up(&self, mask: &Self) -> Self;
    fn filled_files(&self) -> Self;
}

pub(crate) trait BitboardTrait:
    Sized
    + BitOr
    + BitAnd
    + BitXor
    + BitAndAssign
    + BitOrAssign
    + BitXorAssign
    + Not
    + Shl<u8>
    + Shr<u8>
    + Occupied
    + Iterator
{
    fn empty() -> Self;
    fn single(square: Square) -> Self;
    fn count(self) -> u8;
    fn is_empty(&self) -> bool;
    fn contains(self, square: Square) -> bool;
    fn pop(&mut self) -> Option<Square>;
}

mod extended_core;
pub(crate) use extended_core::Bitboard;
