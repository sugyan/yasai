use super::Occupied;
use shogi_core::Square;
use std::arch::aarch64;
use std::mem::MaybeUninit;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

const SINGLE_VALUES: [[u64; 2]; Square::NUM] = {
    let mut values = [[0, 0]; Square::NUM];
    let mut i = 0;
    while i < Square::NUM {
        values[i] = if i < 63 {
            [1 << i, 0]
        } else {
            [0, 1 << (i - 63)]
        };
        i += 1;
    }
    values
};

const MASKED_VALUES: [[u64; 2]; Square::NUM + 2] = {
    let mut values = [[0; 2]; Square::NUM + 2];
    let mut i = 0;
    while i < Square::NUM + 2 {
        let u = (1_u128 << i) - 1;
        values[i] = [u as u64, (u >> 64) as u64];
        i += 1;
    }
    values
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Bitboard(aarch64::uint64x2_t);

impl Bitboard {
    #[inline(always)]
    pub fn empty() -> Self {
        Self(unsafe { aarch64::vdupq_n_u64(0) })
    }
    #[inline(always)]
    pub fn single(square: Square) -> Self {
        let e = SINGLE_VALUES[square.array_index()];
        Self(unsafe { aarch64::vld1q_u64(e.as_ptr()) })
    }
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        unsafe {
            let vqmovn = aarch64::vqmovn_u64(self.0);
            let result = aarch64::vreinterpret_u64_u32(vqmovn);
            aarch64::vget_lane_u64::<0>(result) == 0
        }
    }
    #[inline(always)]
    pub fn contains(&self, square: Square) -> bool {
        !(Self::single(square) & self).is_empty()
    }
    #[inline(always)]
    pub fn count(self) -> u8 {
        let m = self.values();
        (m[0].count_ones() + m[1].count_ones()) as u8
    }
    #[inline(always)]
    fn values(self) -> [u64; 2] {
        unsafe {
            let m = MaybeUninit::<[u64; 2]>::uninit();
            aarch64::vst1q_u64(m.as_ptr() as *mut _, self.0);
            m.assume_init()
        }
    }
    fn sliding_positive(&self, mask: &Bitboard) -> Bitboard {
        let m = (*self & mask).values();
        let tz = if m[0] == 0 {
            (m[1] | 0x0002_0000).trailing_zeros() + 64
        } else {
            m[0].trailing_zeros()
        };
        Self(unsafe {
            aarch64::vandq_u64(
                mask.0,
                aarch64::vld1q_u64(MASKED_VALUES[tz as usize + 1].as_ptr()),
            )
        })
    }
    fn sliding_negative(&self, mask: &Bitboard) -> Bitboard {
        let m = (*self & mask).values();
        let lz = if m[1] == 0 {
            (m[0] | 1).leading_zeros() + 64
        } else {
            m[1].leading_zeros()
        };
        Self(unsafe {
            aarch64::vbicq_u64(
                mask.0,
                aarch64::vld1q_u64(MASKED_VALUES[127 - lz as usize].as_ptr()),
            )
        })
    }
}

impl Occupied for Bitboard {
    #[inline(always)]
    fn shl(&self) -> Self {
        Self(unsafe { aarch64::vshlq_n_u64::<1>(self.0) })
    }
    #[inline(always)]
    fn shr(&self) -> Self {
        Self(unsafe { aarch64::vshrq_n_u64::<1>(self.0) })
    }
    fn sliding_positive_consecutive(&self, mask: &Self) -> Self {
        unsafe {
            let and = aarch64::vandq_u64(self.0, mask.0);
            let all = aarch64::vceqq_u64(self.0, self.0);
            let add = aarch64::vaddq_u64(and, all);
            let xor = aarch64::veorq_u64(add, and);
            Self(aarch64::vandq_u64(xor, mask.0))
        }
    }
    fn sliding_negative_consecutive(&self, mask: &Self) -> Self {
        unsafe {
            let m = aarch64::vandq_u64(self.0, mask.0);
            let m = aarch64::vorrq_u64(m, aarch64::vshrq_n_u64::<1>(m));
            let m = aarch64::vorrq_u64(m, aarch64::vshrq_n_u64::<2>(m));
            let m = aarch64::vorrq_u64(m, aarch64::vshrq_n_u64::<4>(m));
            let m = aarch64::vshrq_n_u64::<1>(m);
            Self(aarch64::vbicq_u64(mask.0, m))
        }
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
        unsafe {
            let mask = aarch64::vld1q_u64([0x4020_1008_0402_0100, 0x0002_0100].as_ptr());
            let sub = aarch64::vsubq_u64(mask, self.0);
            let shr = aarch64::vshrq_n_u64::<8>(aarch64::vandq_u64(sub, mask));
            Self(aarch64::veorq_u64(mask, aarch64::vsubq_u64(mask, shr)))
        }
    }
}

define_bit_trait_unsafe!(
    target_trait => BitAnd, assign_trait => BitAndAssign,
    target_func => bitand, assign_func => bitand_assign,
    intrinsic => aarch64::vandq_u64
);

define_bit_trait_unsafe!(
    target_trait => BitOr, assign_trait => BitOrAssign,
    target_func => bitor, assign_func => bitor_assign,
    intrinsic => aarch64::vorrq_u64
);

define_bit_trait_unsafe!(
    target_trait => BitXor, assign_trait => BitXorAssign,
    target_func => bitxor, assign_func => bitxor_assign,
    intrinsic => aarch64::veorq_u64
);

impl Not for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Bitboard(unsafe {
            aarch64::vbicq_u64(
                aarch64::vld1q_u64([0x7fff_ffff_ffff_ffff, 0x0003_ffff].as_ptr()),
                self.0,
            )
        })
    }
}

impl Not for &Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Bitboard(unsafe {
            aarch64::vbicq_u64(
                aarch64::vld1q_u64([0x7fff_ffff_ffff_ffff, 0x0003_ffff].as_ptr()),
                self.0,
            )
        })
    }
}

impl PartialEq for Bitboard {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        (*self ^ other).is_empty()
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
