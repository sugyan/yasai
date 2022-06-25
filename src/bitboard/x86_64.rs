use super::Occupied;
use shogi_core::Square;
use std::arch::x86_64;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Bitboard(x86_64::__m128i);

const SINGLE_VALUES: [(i64, i64); Square::NUM] = {
    let mut values = [(0, 0); Square::NUM];
    let mut i = 0;
    while i < Square::NUM {
        values[i] = if i < 63 {
            (1 << i, 0)
        } else {
            (0, 1 << (i - 63))
        };
        i += 1;
    }
    values
};

const MASKED_VALUES: [(i64, i64); 16] = [
    (0, 0),
    (0x0000_0000_0000_00ff, 0),
    (0x0000_0000_0000_ffff, 0),
    (0x0000_0000_00ff_ffff, 0),
    (0x0000_0000_ffff_ffff, 0),
    (0x0000_00ff_ffff_ffff, 0),
    (0x0000_ffff_ffff_ffff, 0),
    (0x00ff_ffff_ffff_ffff, 0),
    (-1, 0),
    (-1, 0x0000_0000_0000_00ff),
    (-1, 0x0000_0000_0000_ffff),
    (-1, 0x0000_0000_00ff_ffff),
    (-1, 0x0000_0000_ffff_ffff),
    (-1, 0x0000_00ff_ffff_ffff),
    (-1, 0x0000_ffff_ffff_ffff),
    (-1, 0x00ff_ffff_ffff_ffff),
];

impl Bitboard {
    #[inline(always)]
    pub fn empty() -> Self {
        Self(unsafe { x86_64::_mm_setzero_si128() })
    }
    #[inline(always)]
    pub fn single(square: Square) -> Self {
        let e = SINGLE_VALUES[square.array_index()];
        Self(unsafe { x86_64::_mm_set_epi64x(e.1, e.0) })
    }
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        unsafe { x86_64::_mm_test_all_zeros(self.0, self.0) == 1 }
    }
    #[inline(always)]
    pub fn contains(&self, square: Square) -> bool {
        unsafe { x86_64::_mm_test_all_zeros(self.0, Self::single(square).0) == 0 }
    }
    pub fn pop(&mut self) -> Option<Square> {
        let mut m = {
            unsafe {
                let m = std::mem::MaybeUninit::<[i64; 2]>::uninit();
                x86_64::_mm_storeu_si128(m.as_ptr() as *mut _, self.0);
                m.assume_init()
            }
        };
        if m[0] != 0 {
            unsafe {
                let sq = Some(Square::from_u8_unchecked(Self::pop_lsb(&mut m[0]) + 1));
                self.0 = x86_64::_mm_insert_epi64::<0>(self.0, m[0]);
                sq
            }
        } else if m[1] != 0 {
            unsafe {
                let sq = Some(Square::from_u8_unchecked(Self::pop_lsb(&mut m[1]) + 64));
                self.0 = x86_64::_mm_insert_epi64::<1>(self.0, m[1]);
                sq
            }
        } else {
            None
        }
    }
    fn pop_lsb(n: &mut i64) -> u8 {
        let ret = n.trailing_zeros() as u8;
        *n = *n & (*n - 1);
        ret
    }
}

impl Occupied for Bitboard {
    #[inline(always)]
    fn shl(&self) -> Self {
        Self(unsafe { x86_64::_mm_slli_epi64::<1>(self.0) })
    }
    #[inline(always)]
    fn shr(&self) -> Self {
        Self(unsafe { x86_64::_mm_srli_epi64::<1>(self.0) })
    }
    fn sliding_positive_consecutive(&self, mask: &Self) -> Self {
        unsafe {
            let masked = x86_64::_mm_and_si128(self.0, mask.0);
            // calculate decremented masked
            let all = x86_64::_mm_cmpeq_epi64(self.0, self.0);
            //      self.0: ...00000000000010000000 0000000000000000000000000000000000000000000000000000000000000000
            let add = x86_64::_mm_add_epi64(masked, all);
            // self.0 + !0: ...00000000000001111111 1111111111111111111111111111111111111111111111111111111111111111
            let cmp = x86_64::_mm_cmpeq_epi64(add, all);
            // self.0 == 0: ...00000000000000000000 1111111111111111111111111111111111111111111111111111111111111111
            let shl = x86_64::_mm_slli_si128::<8>(x86_64::_mm_xor_si128(cmp, all));
            //  !cmp << 64: ...00000000000000000000 0000000000000000000000000000000000000000000000000000000000000000
            let dec = x86_64::_mm_sub_epi64(add, shl);
            //   add + shl: ...00000000000001111111 1111111111111111111111111111111111111111111111111111111111111111
            Self(x86_64::_mm_and_si128(
                x86_64::_mm_xor_si128(masked, dec),
                mask.0,
            ))
        }
    }
    fn sliding_negative_consecutive(&self, mask: &Self) -> Self {
        unsafe {
            let masked = x86_64::_mm_and_si128(self.0, mask.0);
            let m = masked;
            let m = x86_64::_mm_or_si128(m, x86_64::_mm_srli_epi64::<1>(m));
            let m = x86_64::_mm_or_si128(m, x86_64::_mm_srli_epi64::<2>(m));
            let m = x86_64::_mm_or_si128(m, x86_64::_mm_srli_epi64::<4>(m));
            let m = x86_64::_mm_srli_epi64::<1>(m);
            Self(x86_64::_mm_andnot_si128(m, mask.0))
        }
    }
    fn sliding_positives(&self, masks: &[Self; 2]) -> Self {
        unsafe {
            let self256 = x86_64::_mm256_broadcastsi128_si256(self.0);
            let mask256 = x86_64::_mm256_set_m128i(masks[0].0, masks[1].0);
            let masked = x86_64::_mm256_and_si256(self256, mask256);
            // decrement masked 256
            let all = x86_64::_mm256_cmpeq_epi64(self256, self256);
            let add = x86_64::_mm256_add_epi64(masked, all);
            let cmp = x86_64::_mm256_cmpeq_epi64(add, all);
            let shl = x86_64::_mm256_slli_si256::<8>(x86_64::_mm256_xor_si256(cmp, all));
            let dec = x86_64::_mm256_sub_epi64(add, shl);
            // (masked ^ masked.decrement()) & mask
            let xor = x86_64::_mm256_xor_si256(masked, dec);
            let ret = x86_64::_mm256_and_si256(xor, mask256);
            Self(x86_64::_mm_or_si128(
                x86_64::_mm256_castsi256_si128(ret),
                x86_64::_mm256_extracti128_si256::<1>(ret),
            ))
        }
    }
    fn sliding_negatives(&self, masks: &[Self; 2]) -> Self {
        unsafe {
            let self256 = x86_64::_mm256_broadcastsi128_si256(self.0);
            let mask256 = x86_64::_mm256_set_m128i(masks[0].0, masks[1].0);
            let masked = x86_64::_mm256_and_si256(self256, mask256);

            let eq = x86_64::_mm256_cmpeq_epi8(masked, x86_64::_mm256_setzero_si256());
            let mv = x86_64::_mm256_movemask_epi8(eq) as u32;
            let e0 = MASKED_VALUES[15 - (mv ^ 0xffff_ffff | 0x0001_0000).leading_zeros() as usize];
            let e1 = MASKED_VALUES[31 - (mv & 0xffff ^ 0xffff | 0x0001).leading_zeros() as usize];

            let m = masked;
            let m = x86_64::_mm256_or_si256(m, x86_64::_mm256_srli_epi16::<1>(m));
            let m = x86_64::_mm256_or_si256(m, x86_64::_mm256_srli_epi16::<2>(m));
            let m = x86_64::_mm256_or_si256(m, x86_64::_mm256_srli_epi16::<4>(m));
            let m = x86_64::_mm256_or_si256(
                x86_64::_mm256_srli_epi16::<1>(m),
                x86_64::_mm256_set_epi64x(e0.1, e0.0, e1.1, e1.0),
            );
            let ret = x86_64::_mm256_andnot_si256(m, mask256);
            Self(x86_64::_mm_or_si128(
                x86_64::_mm256_castsi256_si128(ret),
                x86_64::_mm256_extracti128_si256::<1>(ret),
            ))
        }
    }
    fn vacant_files(&self) -> Self {
        unsafe {
            let mask = x86_64::_mm_set_epi64x(0x0002_0100, 0x4020_1008_0402_0100);
            let sub = x86_64::_mm_sub_epi64(mask, self.0);
            let shr = x86_64::_mm_srli_epi64::<8>(x86_64::_mm_and_si128(mask, sub));
            Self(x86_64::_mm_xor_si128(
                mask,
                x86_64::_mm_sub_epi64(mask, shr),
            ))
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
                Self(unsafe { x86_64::$intrinsic(self.0, rhs.0) })
            }
        }
        impl $trait<&Bitboard> for Bitboard {
            type Output = Bitboard;

            #[inline(always)]
            fn $func(self, rhs: &Self) -> Self::Output {
                Self(unsafe { x86_64::$intrinsic(self.0, rhs.0) })
            }
        }
        impl $assign_trait for Bitboard {
            #[inline(always)]
            fn $assign_func(&mut self, rhs: Self) {
                self.0 = unsafe { x86_64::$intrinsic(self.0, rhs.0) }
            }
        }
    };
}

define_bit_trait!(
    target_trait => BitAnd, assign_trait => BitAndAssign,
    target_func => bitand, assign_func => bitand_assign,
    intrinsic => _mm_and_si128
);

define_bit_trait!(
    target_trait => BitOr, assign_trait => BitOrAssign,
    target_func => bitor, assign_func => bitor_assign,
    intrinsic => _mm_or_si128
);

define_bit_trait!(
    target_trait => BitXor, assign_trait => BitXorAssign,
    target_func => bitxor, assign_func => bitxor_assign,
    intrinsic => _mm_xor_si128
);

impl Not for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Bitboard(unsafe {
            x86_64::_mm_andnot_si128(
                self.0,
                x86_64::_mm_set_epi64x(0x0003_ffff, 0x7fff_ffff_ffff_ffff),
            )
        })
    }
}

impl Not for &Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Bitboard(unsafe {
            x86_64::_mm_andnot_si128(
                self.0,
                x86_64::_mm_set_epi64x(0x0003_ffff, 0x7fff_ffff_ffff_ffff),
            )
        })
    }
}

impl PartialEq for Bitboard {
    fn eq(&self, other: &Self) -> bool {
        // https://stackoverflow.com/a/26883316
        unsafe {
            let xor = x86_64::_mm_xor_si128(self.0, other.0);
            x86_64::_mm_test_all_zeros(xor, xor) == 1
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

#[cfg(test)]
mod tests {
    use super::*;
    use shogi_core::consts::square::*;

    #[test]
    fn sliding_positives() {
        let bb = Bitboard::single(SQ_8C) | Bitboard::single(SQ_8G);
        assert_eq!(
            bb | Bitboard::single(SQ_7D) | Bitboard::single(SQ_7F),
            bb.sliding_positives(&[
                Bitboard::single(SQ_7D) | Bitboard::single(SQ_8C) | Bitboard::single(SQ_9B),
                Bitboard::single(SQ_7F) | Bitboard::single(SQ_8G) | Bitboard::single(SQ_9H),
            ])
        );
    }

    #[test]
    fn sliding_negatives() {
        let bb = Bitboard::single(SQ_2C) | Bitboard::single(SQ_2G);
        assert_eq!(
            bb | Bitboard::single(SQ_3D) | Bitboard::single(SQ_3F),
            bb.sliding_negatives(&[
                Bitboard::single(SQ_3D) | Bitboard::single(SQ_2C) | Bitboard::single(SQ_1B),
                Bitboard::single(SQ_3F) | Bitboard::single(SQ_2G) | Bitboard::single(SQ_1H),
            ])
        );
    }
}
