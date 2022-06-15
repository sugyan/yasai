use super::{BitboardTrait, Occupied};
use shogi_core::Square;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, Shr};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Bitboard(shogi_core::Bitboard);

impl BitboardTrait for Bitboard {
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
    fn pop(&mut self) -> Option<Square> {
        self.0.pop()
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl Shl<u8> for Bitboard {
    type Output = Self;

    fn shl(self, rhs: u8) -> Self::Output {
        Self(unsafe { self.0.shift_down(rhs) })
    }
}

impl Shr<u8> for Bitboard {
    type Output = Self;

    fn shr(self, rhs: u8) -> Self::Output {
        Self(unsafe { self.0.shift_up(rhs) })
    }
}

//                                                            0001 _ 1111 1110 1111 1111
// 0011 1111 1101 1111 _ 1110 1111 1111 0111 _ 1111 1011 1111 1101 _ 1111 1110 1111 1111
const FILL_MASK_VALUE: u128 = 0x0001_feff_3fdf_eff7_fbfd_feff;

impl Occupied for Bitboard {
    fn sliding_left_down(&self, mask: &Self) -> Self {
        let bb = *self & *mask;
        if bb.is_empty() {
            return *mask;
        }
        let tz = bb.0.to_u128().trailing_zeros() as u8;
        *mask & Self(unsafe { shogi_core::Bitboard::from_u128_unchecked((1 << (tz + 1)) - 1) })
    }
    fn sliding_right_up(&self, mask: &Self) -> Self {
        let bb = *self & *mask;
        if bb.is_empty() {
            return *mask;
        }
        let lz = bb.0.to_u128().leading_zeros() as u8;
        *mask & Self(unsafe { shogi_core::Bitboard::from_u128_unchecked(!((1 << (127 - lz)) - 1)) })
    }
    fn filled_files(&self) -> Self {
        let mask = Self(unsafe { shogi_core::Bitboard::from_u128_unchecked(FILL_MASK_VALUE) });
        let bb = Self(unsafe {
            shogi_core::Bitboard::from_u128_unchecked(
                self.0.to_u128() + FILL_MASK_VALUE + (1 << 63),
            )
        });
        mask ^ Self(unsafe {
            shogi_core::Bitboard::from_u128_unchecked((bb >> 8).0.to_u128() + FILL_MASK_VALUE)
        })
    }
}

impl Iterator for Bitboard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
