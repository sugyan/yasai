use super::BitboardTrait;
use shogi_core::Square;
use std::ops::{
    Add, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, Shr, Sub,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct ExtendedCoreBitboard(shogi_core::Bitboard);

impl ExtendedCoreBitboard {
    const MASK0: u64 = (1 << 63) - 1;
    const MASK1: u64 = (1 << 18) - 1;
}

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

impl Add for ExtendedCoreBitboard {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let repr = self.to_u128() + rhs.to_u128() + (1 << 63);
        let values = (repr as u64 & Self::MASK0, (repr >> 64) as u64 & Self::MASK1);
        unsafe { Self::from_u128_unchecked(values.0 as u128 | (values.1 as u128) << 64) }
    }
}

impl Sub for ExtendedCoreBitboard {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let repr = (self.to_u128() | 0x0004_0000_8000_0000_0000_0000) - rhs.to_u128();
        let mut values = (repr as u64 & Self::MASK0, (repr >> 64) as u64 & Self::MASK1);
        if repr & (1 << 63) == 0 {
            values.1 -= 1;
        }
        unsafe { Self::from_u128_unchecked(values.0 as u128 | (values.1 as u128) << 64) }
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

#[cfg(test)]
mod tests {
    use super::*;
    use shogi_core::consts::square::*;

    #[test]
    fn add() {
        let bb =
            unsafe { ExtendedCoreBitboard::from_u128_unchecked(0x0002_0100_4020_1008_0402_0100) };
        assert_eq!(
            vec![SQ_1I, SQ_2I, SQ_3I, SQ_4I, SQ_5I, SQ_6I, SQ_7I, SQ_8I, SQ_9I],
            bb.collect::<Vec<_>>()
        );
        assert_eq!(
            vec![SQ_2A, SQ_2I, SQ_3I, SQ_4I, SQ_5I, SQ_6I, SQ_7I, SQ_8I, SQ_9I],
            (bb + ExtendedCoreBitboard::single(SQ_1I)).collect::<Vec<_>>()
        );
        assert_eq!(
            vec![SQ_1I, SQ_3A, SQ_3I, SQ_4I, SQ_5I, SQ_6I, SQ_7I, SQ_8I, SQ_9I],
            (bb + ExtendedCoreBitboard::single(SQ_2I)).collect::<Vec<_>>()
        );
        assert_eq!(
            vec![SQ_1I, SQ_2I, SQ_4A, SQ_4I, SQ_5I, SQ_6I, SQ_7I, SQ_8I, SQ_9I],
            (bb + ExtendedCoreBitboard::single(SQ_3I)).collect::<Vec<_>>()
        );
        assert_eq!(
            vec![SQ_1I, SQ_2I, SQ_3I, SQ_5A, SQ_5I, SQ_6I, SQ_7I, SQ_8I, SQ_9I],
            (bb + ExtendedCoreBitboard::single(SQ_4I)).collect::<Vec<_>>()
        );
        assert_eq!(
            vec![SQ_1I, SQ_2I, SQ_3I, SQ_4I, SQ_6A, SQ_6I, SQ_7I, SQ_8I, SQ_9I],
            (bb + ExtendedCoreBitboard::single(SQ_5I)).collect::<Vec<_>>()
        );
        assert_eq!(
            vec![SQ_1I, SQ_2I, SQ_3I, SQ_4I, SQ_5I, SQ_7A, SQ_7I, SQ_8I, SQ_9I],
            (bb + ExtendedCoreBitboard::single(SQ_6I)).collect::<Vec<_>>()
        );
        assert_eq!(
            vec![SQ_1I, SQ_2I, SQ_3I, SQ_4I, SQ_5I, SQ_6I, SQ_8A, SQ_8I, SQ_9I],
            (bb + ExtendedCoreBitboard::single(SQ_7I)).collect::<Vec<_>>()
        );
        assert_eq!(
            vec![SQ_1I, SQ_2I, SQ_3I, SQ_4I, SQ_5I, SQ_6I, SQ_7I, SQ_9A, SQ_9I],
            (bb + ExtendedCoreBitboard::single(SQ_8I)).collect::<Vec<_>>()
        );
        assert_eq!(
            vec![SQ_1I, SQ_2I, SQ_3I, SQ_4I, SQ_5I, SQ_6I, SQ_7I, SQ_8I],
            (bb + ExtendedCoreBitboard::single(SQ_9I)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn sub() {
        let bb = unsafe { ExtendedCoreBitboard::from_u128_unchecked(0x0201_0040_2010_0804_0201) };
        assert_eq!(
            vec![SQ_1A, SQ_2A, SQ_3A, SQ_4A, SQ_5A, SQ_6A, SQ_7A, SQ_8A, SQ_9A],
            bb.collect::<Vec<_>>()
        );
        assert_eq!(
            vec![SQ_1A, SQ_1I, SQ_3A, SQ_4A, SQ_5A, SQ_6A, SQ_7A, SQ_8A, SQ_9A],
            (bb - ExtendedCoreBitboard::single(SQ_1I)).collect::<Vec<_>>()
        );
        assert_eq!(
            vec![SQ_1A, SQ_2A, SQ_2I, SQ_4A, SQ_5A, SQ_6A, SQ_7A, SQ_8A, SQ_9A],
            (bb - ExtendedCoreBitboard::single(SQ_2I)).collect::<Vec<_>>()
        );
        assert_eq!(
            vec![SQ_1A, SQ_2A, SQ_3A, SQ_3I, SQ_5A, SQ_6A, SQ_7A, SQ_8A, SQ_9A],
            (bb - ExtendedCoreBitboard::single(SQ_3I)).collect::<Vec<_>>()
        );
        assert_eq!(
            vec![SQ_1A, SQ_2A, SQ_3A, SQ_4A, SQ_4I, SQ_6A, SQ_7A, SQ_8A, SQ_9A],
            (bb - ExtendedCoreBitboard::single(SQ_4I)).collect::<Vec<_>>()
        );
        assert_eq!(
            vec![SQ_1A, SQ_2A, SQ_3A, SQ_4A, SQ_5A, SQ_5I, SQ_7A, SQ_8A, SQ_9A],
            (bb - ExtendedCoreBitboard::single(SQ_5I)).collect::<Vec<_>>()
        );
        assert_eq!(
            vec![SQ_1A, SQ_2A, SQ_3A, SQ_4A, SQ_5A, SQ_6A, SQ_6I, SQ_8A, SQ_9A],
            (bb - ExtendedCoreBitboard::single(SQ_6I)).collect::<Vec<_>>()
        );
        assert_eq!(
            vec![SQ_1A, SQ_2A, SQ_3A, SQ_4A, SQ_5A, SQ_6A, SQ_7A, SQ_7I, SQ_9A],
            (bb - ExtendedCoreBitboard::single(SQ_7I)).collect::<Vec<_>>()
        );
        assert_eq!(
            vec![SQ_1A, SQ_2A, SQ_3A, SQ_4A, SQ_5A, SQ_6A, SQ_7A, SQ_8A, SQ_8I],
            (bb - ExtendedCoreBitboard::single(SQ_8I)).collect::<Vec<_>>()
        );
        assert_eq!(
            vec![SQ_1A, SQ_2A, SQ_3A, SQ_4A, SQ_5A, SQ_6A, SQ_7A, SQ_8A, SQ_9A, SQ_9I],
            (bb - ExtendedCoreBitboard::single(SQ_9I)).collect::<Vec<_>>()
        );
    }
}
