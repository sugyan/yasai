use super::BitboardTrait;
use shogi_core::Square;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, Shr};

#[derive(Clone, Copy, Debug)]
pub(crate) struct ExtendedCoreBitboard(shogi_core::Bitboard);

impl BitboardTrait for ExtendedCoreBitboard {
    fn empty() -> Self {
        Self(shogi_core::Bitboard::empty())
    }
    fn single(square: Square) -> Self {
        Self(shogi_core::Bitboard::single(square))
    }
    fn count(self) -> u8 {
        self.0.count()
    }
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    fn contains(self, square: Square) -> bool {
        self.0.contains(square)
    }
    fn flip(self) -> Self {
        Self(self.0.flip())
    }
    fn pop(&mut self) -> Option<Square> {
        self.0.pop()
    }
    fn to_u128(self) -> u128 {
        self.0.to_u128()
    }
    unsafe fn from_u128_unchecked(a: u128) -> Self {
        Self(shogi_core::Bitboard::from_u128_unchecked(a))
    }
}

impl BitAnd for ExtendedCoreBitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitOr for ExtendedCoreBitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitXor for ExtendedCoreBitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl BitAndAssign for ExtendedCoreBitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOrAssign for ExtendedCoreBitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXorAssign for ExtendedCoreBitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl Not for ExtendedCoreBitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl Shl<u8> for ExtendedCoreBitboard {
    type Output = Self;

    fn shl(self, rhs: u8) -> Self::Output {
        Self(unsafe { self.0.shift_down(rhs) })
    }
}

impl Shr<u8> for ExtendedCoreBitboard {
    type Output = Self;

    fn shr(self, rhs: u8) -> Self::Output {
        Self(unsafe { self.0.shift_up(rhs) })
    }
}

impl Iterator for ExtendedCoreBitboard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
