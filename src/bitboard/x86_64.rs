use super::Occupied;
use shogi_core::Square;
use std::arch::x86_64;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, Shr};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Bitboard(x86_64::__m128i);

impl Bitboard {
    #[inline(always)]
    pub fn empty() -> Self {
        Self(unsafe { x86_64::_mm_setzero_si128() })
    }
    #[inline(always)]
    pub const fn single(square: Square) -> Self {
        todo!()
    }
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        unsafe { x86_64::_mm_test_all_zeros(self.0, self.0) == 1 }
    }
    pub fn contains(self, square: Square) -> bool {
        todo!()
    }
    pub const unsafe fn shift_down(self, delta: u8) -> Self {
        todo!()
    }
    pub const unsafe fn shift_up(self, delta: u8) -> Self {
        todo!()
    }
    pub fn pop(&mut self) -> Option<Square> {
        todo!()
    }
}

impl Occupied for Bitboard {
    fn shl(&self, rhs: u8) -> Self {
        todo!()
    }
    fn shr(&self, rhs: u8) -> Self {
        todo!()
    }
    fn sliding_positive(&self, mask: &Self) -> Self {
        todo!()
    }

    fn sliding_negative(&self, mask: &Self) -> Self {
        todo!()
    }

    fn sliding_positives(&self, masks: &[Self; 2]) -> Self {
        todo!()
    }

    fn sliding_negatives(&self, masks: &[Self; 2]) -> Self {
        todo!()
    }

    fn vacant_files(&self) -> Self {
        todo!()
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        todo!()
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        todo!()
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        todo!()
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        todo!()
    }
}

impl BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self {
        todo!()
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        todo!()
    }
}

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self {
        todo!()
    }
}

impl Iterator for Bitboard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        self.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(81))
    }
}
