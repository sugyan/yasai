use super::Occupied;
use shogi_core::Square;
use std::arch::wasm32;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

const SINGLES: [wasm32::v128; Square::NUM] = {
    let mut values = [ZERO; Square::NUM];
    let mut i = 0;
    while i < Square::NUM {
        values[i] = if i < 63 {
            wasm32::u64x2(1 << i, 0)
        } else {
            wasm32::u64x2(0, 1 << (i - 63))
        };
        i += 1;
    }
    values
};

const MASKED_VALUES: [wasm32::v128; Square::NUM + 2] = {
    let mut values = [ZERO; Square::NUM + 2];
    let mut i = 0;
    while i < Square::NUM + 2 {
        let u = (1_u128 << i) - 1;
        values[i] = wasm32::u64x2(u as u64, (u >> 64) as u64);
        i += 1;
    }
    values
};

const ZERO: wasm32::v128 = wasm32::u64x2(0, 0);
const ONES: wasm32::v128 = wasm32::u64x2(0x7fff_ffff_ffff_ffff, 0x0003_ffff);

#[derive(Clone, Copy, Debug)]
pub(crate) struct Bitboard(wasm32::v128);

impl Bitboard {
    #[inline(always)]
    pub fn empty() -> Self {
        Self(ZERO)
    }
    #[inline(always)]
    pub fn single(square: Square) -> Self {
        Self(SINGLES[square.array_index()])
    }
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        !wasm32::v128_any_true(self.0)
    }
    #[inline(always)]
    pub fn contains(&self, square: Square) -> bool {
        wasm32::v128_any_true(wasm32::v128_and(self.0, Self::single(square).0))
    }
    #[inline(always)]
    pub fn count(self) -> u8 {
        let m = self.values();
        (m[0].count_ones() + m[1].count_ones()) as u8
    }
    #[inline(always)]
    fn values(self) -> [u64; 2] {
        [
            wasm32::u64x2_extract_lane::<0>(self.0),
            wasm32::u64x2_extract_lane::<1>(self.0),
        ]
    }
    fn sliding_positive(&self, mask: &Bitboard) -> Bitboard {
        let m = (*self & mask).values();
        let tz = if m[0] == 0 {
            (m[1] | 0x0002_0000).trailing_zeros() + 64
        } else {
            m[0].trailing_zeros()
        };
        Self(wasm32::v128_and(mask.0, MASKED_VALUES[tz as usize + 1]))
    }
    fn sliding_negative(&self, mask: &Bitboard) -> Bitboard {
        let m = (*self & mask).values();
        let lz = if m[1] == 0 {
            (m[0] | 1).leading_zeros() + 64
        } else {
            m[1].leading_zeros()
        };
        Self(wasm32::v128_andnot(
            mask.0,
            MASKED_VALUES[127 - lz as usize],
        ))
    }
}

impl Occupied for Bitboard {
    #[inline(always)]
    fn shl(&self) -> Self {
        Self(wasm32::u64x2_shl(self.0, 1))
    }
    #[inline(always)]
    fn shr(&self) -> Self {
        Self(wasm32::u64x2_shr(self.0, 1))
    }
    #[inline(always)]
    fn sliding_positive_consecutive(&self, mask: &Self) -> Self {
        let and = wasm32::v128_and(self.0, mask.0);
        let all = wasm32::u64x2_eq(self.0, self.0);
        let add = wasm32::u64x2_add(and, all);
        let xor = wasm32::v128_xor(add, and);
        Self(wasm32::v128_and(xor, mask.0))
    }
    #[inline(always)]
    fn sliding_negative_consecutive(&self, mask: &Self) -> Self {
        self.sliding_negative(mask)
    }
    #[inline(always)]
    fn sliding_positives(&self, masks: &[Self; 2]) -> Self {
        self.sliding_positive(&masks[0]) | self.sliding_positive(&masks[1])
    }
    #[inline(always)]
    fn sliding_negatives(&self, masks: &[Self; 2]) -> Self {
        self.sliding_negative(&masks[0]) | self.sliding_negative(&masks[1])
    }
    fn vacant_files(&self) -> Self {
        let mask = wasm32::u64x2(0x4020_1008_0402_0100, 0x0002_0100);
        let sub = wasm32::u64x2_sub(mask, self.0);
        let and = wasm32::v128_and(sub, mask);
        let shr = wasm32::u64x2_shr(and, 8);
        Self(wasm32::v128_xor(mask, wasm32::u64x2_sub(mask, shr)))
    }
}

define_bit_trait!(
    target_trait => BitAnd, assign_trait => BitAndAssign,
    target_func => bitand, assign_func => bitand_assign,
    intrinsic => wasm32::v128_and
);

define_bit_trait!(
    target_trait => BitOr, assign_trait => BitOrAssign,
    target_func => bitor, assign_func => bitor_assign,
    intrinsic => wasm32::v128_or
);

define_bit_trait!(
    target_trait => BitXor, assign_trait => BitXorAssign,
    target_func => bitxor, assign_func => bitxor_assign,
    intrinsic => wasm32::v128_xor
);

impl Not for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Bitboard(wasm32::v128_andnot(ONES, self.0))
    }
}

impl Not for &Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Bitboard(wasm32::v128_andnot(ONES, self.0))
    }
}

impl PartialEq for Bitboard {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        wasm32::u64x2_all_true(wasm32::u64x2_eq(self.0, other.0))
    }
}

pub(crate) struct SquareIterator([u64; 2]);

impl SquareIterator {
    #[inline(always)]
    fn pop_lsb(n: &mut u64) -> u8 {
        let pos = n.trailing_zeros() as u8;
        *n &= n.wrapping_sub(1);
        pos
    }
}

impl Iterator for SquareIterator {
    type Item = Square;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.0[0] != 0 {
            return Some(unsafe { Square::from_u8_unchecked(Self::pop_lsb(&mut self.0[0]) + 1) });
        }
        if self.0[1] != 0 {
            return Some(unsafe { Square::from_u8_unchecked(Self::pop_lsb(&mut self.0[1]) + 64) });
        }
        None
    }
}

impl IntoIterator for Bitboard {
    type Item = Square;
    type IntoIter = SquareIterator;

    fn into_iter(self) -> Self::IntoIter {
        SquareIterator(self.values())
    }
}
