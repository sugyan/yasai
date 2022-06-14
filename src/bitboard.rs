use shogi_core::Square;
use std::ops::{
    Add, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, Shr, Sub,
};

pub trait BitboardTrait:
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
    + Add
    + Sub
    + Iterator
{
    fn empty() -> Self;
    fn single(square: Square) -> Self;
    fn count(self) -> u8;
    fn is_empty(&self) -> bool;
    fn contains(self, square: Square) -> bool;
    fn flip(self) -> Self;
    fn pop(&mut self) -> Option<Square>;
    fn to_u128(self) -> u128;
    unsafe fn from_u128_unchecked(a: u128) -> Self;
}

mod extended_core;
pub(crate) use extended_core::Bitboard;
