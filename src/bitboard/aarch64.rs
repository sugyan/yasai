use super::Occupied;
use shogi_core::Square;
use std::arch::aarch64;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Bitboard(aarch64::uint64x2_t);

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

const MASKED_VALUES: [u128; Square::NUM + 2] = {
    let mask = 0x0003_ffff_7fff_ffff_ffff_ffff;
    let mut bbs = [0; Square::NUM + 2];
    let mut i = 0;
    while i < Square::NUM + 2 {
        bbs[i] = mask & ((1 << i) - 1);
        i += 1;
    }
    bbs
};

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
            aarch64::vget_lane_u64::<0>(aarch64::vreinterpret_u64_u32(aarch64::vqmovn_u64(
                aarch64::veorq_u64(self.0, aarch64::vdupq_n_u64(0)),
            ))) == 0
        }
    }
    #[inline(always)]
    pub fn contains(&self, square: Square) -> bool {
        unsafe {
            aarch64::vget_lane_u64::<0>(aarch64::vreinterpret_u64_u32(aarch64::vqmovn_u64(
                aarch64::vandq_u64(self.0, Self::single(square).0),
            ))) != 0
        }
    }
    pub fn pop(&mut self) -> Option<Square> {
        let mut m = unsafe {
            let mut m = std::mem::MaybeUninit::<[u64; 2]>::uninit();
            aarch64::vst1q_u64(m.as_mut_ptr() as *mut _, self.0);
            m.assume_init()
        };
        if m[0] != 0 {
            unsafe {
                let sq = Some(Square::from_u8_unchecked(Self::pop_lsb(&mut m[0]) + 1));
                self.0 = aarch64::vsetq_lane_u64::<0>(m[0], self.0);
                sq
            }
        } else if m[1] != 0 {
            unsafe {
                let sq = Some(Square::from_u8_unchecked(Self::pop_lsb(&mut m[1]) + 64));
                self.0 = aarch64::vsetq_lane_u64::<1>(m[1], self.0);
                sq
            }
        } else {
            None
        }
    }
    fn pop_lsb(n: &mut u64) -> u8 {
        let ret = n.trailing_zeros() as u8;
        *n = *n & (*n - 1);
        ret
    }

    fn to_u128(self) -> u128 {
        unsafe {
            let mut m = std::mem::MaybeUninit::<[u64; 2]>::uninit();
            aarch64::vst1q_u64(m.as_mut_ptr() as *mut _, self.0);
            let m = m.assume_init();
            (m[1] as u128) << 64 | m[0] as u128
        }
    }
    fn sliding_positive(&self, mask: &Bitboard) -> Bitboard {
        let tz = ((*self & *mask).to_u128() | 1 << 81).trailing_zeros();
        let m = MASKED_VALUES[tz as usize + 1];
        *mask & Self(unsafe { aarch64::vld1q_u64([m as u64, (m >> 64) as u64].as_ptr()) })
    }
    fn sliding_negative(&self, mask: &Bitboard) -> Bitboard {
        let lz = ((*self & *mask).to_u128() | 1).leading_zeros();
        let m = MASKED_VALUES[127 - lz as usize];
        *mask & !Self(unsafe { aarch64::vld1q_u64([m as u64, (m >> 64) as u64].as_ptr()) })
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
        self.sliding_positive(mask)
    }
    fn sliding_negative_consecutive(&self, mask: &Self) -> Self {
        self.sliding_negative(mask)
    }
    fn sliding_positives(&self, masks: &[Self; 2]) -> Self {
        self.sliding_positive(&masks[0]) | self.sliding_positive(&masks[1])
    }
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

macro_rules! define_bit_trait {
    (
        target_trait => $trait:ident, assign_trait => $assign_trait:ident,
        target_func  => $func:ident,  assign_func  => $assign_func:ident,
        intrinsic    => $intrinsic:ident
    ) => {
        impl $trait for Bitboard {
            type Output = Bitboard;

            #[inline(always)]
            fn $func(self, rhs: Self) -> Self::Output {
                Self(unsafe { aarch64::$intrinsic(self.0, rhs.0) })
            }
        }
        impl $trait<&Bitboard> for Bitboard {
            type Output = Bitboard;

            #[inline(always)]
            fn $func(self, rhs: &Self) -> Self::Output {
                Self(unsafe { aarch64::$intrinsic(self.0, rhs.0) })
            }
        }
        impl $assign_trait for Bitboard {
            #[inline(always)]
            fn $assign_func(&mut self, rhs: Self) {
                self.0 = unsafe { aarch64::$intrinsic(self.0, rhs.0) }
            }
        }
    };
}

define_bit_trait!(
    target_trait => BitAnd, assign_trait => BitAndAssign,
    target_func => bitand, assign_func => bitand_assign,
    intrinsic => vandq_u64
);

define_bit_trait!(
    target_trait => BitOr, assign_trait => BitOrAssign,
    target_func => bitor, assign_func => bitor_assign,
    intrinsic => vorrq_u64
);

define_bit_trait!(
    target_trait => BitXor, assign_trait => BitXorAssign,
    target_func => bitxor, assign_func => bitxor_assign,
    intrinsic => veorq_u64
);

impl Not for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Bitboard(unsafe {
            aarch64::vandq_u64(
                aarch64::vreinterpretq_u64_u32(aarch64::vmvnq_u32(aarch64::vreinterpretq_u32_u64(
                    self.0,
                ))),
                aarch64::vld1q_u64([0x7fff_ffff_ffff_ffff, 0x0003_ffff].as_ptr()),
            )
        })
    }
}

impl Not for &Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Bitboard(unsafe {
            aarch64::vandq_u64(
                aarch64::vreinterpretq_u64_u32(aarch64::vmvnq_u32(aarch64::vreinterpretq_u32_u64(
                    self.0,
                ))),
                aarch64::vld1q_u64([0x7fff_ffff_ffff_ffff, 0x0003_ffff].as_ptr()),
            )
        })
    }
}

impl PartialEq for Bitboard {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            aarch64::vget_lane_u64::<0>(aarch64::vreinterpret_u64_u32(aarch64::vqmovn_u64(
                aarch64::veorq_u64(self.0, other.0),
            ))) == 0
        }
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
