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

const MASKED_VALUES: [(i64, i64); Square::NUM + 2] = {
    let mut values = [(0, 0); Square::NUM + 2];
    let mut i = 0;
    while i < Square::NUM + 2 {
        let value = (1_u128 << i) - 1;
        values[i] = (value as i64, (value >> 64) as i64);
        i += 1;
    }
    values
};

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
        let mut m = self.to_i64x2();
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
    fn decrement(&self) -> Self {
        unsafe {
            //      self.0: ...00000000000010000000 0000000000000000000000000000000000000000000000000000000000000000
            let sub = x86_64::_mm_sub_epi64(self.0, x86_64::_mm_set_epi64x(0, 1));
            //  self.0 - 1: ...00000000000010000000 1111111111111111111111111111111111111111111111111111111111111111
            let cmp = x86_64::_mm_cmpeq_epi64(self.0, x86_64::_mm_setzero_si128());
            // self.0 == 0: ...00000000000000000000 1111111111111111111111111111111111111111111111111111111111111111
            let shl = x86_64::_mm_slli_si128::<8>(cmp);
            //   cmp << 64: ...11111111111111111111 0000000000000000000000000000000000000000000000000000000000000000
            let add = x86_64::_mm_add_epi64(sub, shl);
            //   sub + shl: ...00000000000001111111 1111111111111111111111111111111111111111111111111111111111111111
            Self(add)
        }
    }
    fn pop_lsb(n: &mut i64) -> u8 {
        let ret = n.trailing_zeros() as u8;
        *n = *n & (*n - 1);
        ret
    }
    fn to_i64x2(self) -> [i64; 2] {
        unsafe {
            let m = std::mem::MaybeUninit::<[i64; 2]>::uninit();
            x86_64::_mm_storeu_si128(m.as_ptr() as *mut _, self.0);
            m.assume_init()
        }
    }
    fn leading_zeros(&self) -> u32 {
        let m = self.to_i64x2();
        if m[1] == 0 {
            m[0].leading_zeros() + 64
        } else {
            m[1].leading_zeros()
        }
    }
}

impl Occupied for Bitboard {
    fn shl(&self) -> Self {
        Self(unsafe { x86_64::_mm_slli_epi64::<1>(self.0) })
    }
    fn shr(&self) -> Self {
        Self(unsafe { x86_64::_mm_srli_epi64::<1>(self.0) })
    }
    fn sliding_positive(&self, mask: &Self) -> Self {
        let masked = *self & mask;
        (masked ^ masked.decrement()) & mask
    }
    fn sliding_negative(&self, mask: &Self) -> Self {
        let lz = (*self & *mask | Self::single(Square::SQ_1A)).leading_zeros();
        let e = MASKED_VALUES[127 - lz as usize];
        *mask & !Self(unsafe { x86_64::_mm_set_epi64x(e.1, e.0) })
    }
    fn sliding_positives(&self, masks: &[Self; 2]) -> Self {
        self.sliding_positive(&masks[0]) | self.sliding_positive(&masks[1])
    }
    fn sliding_negatives(&self, masks: &[Self; 2]) -> Self {
        self.sliding_negative(&masks[0]) | self.sliding_negative(&masks[1])
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

    #[test]
    fn decrement() {
        {
            let bb = Bitboard::single(Square::SQ_1B);
            assert_eq!(Bitboard::single(Square::SQ_1A), bb.decrement());
        }
        {
            let bb = Bitboard::single(Square::SQ_1C);
            assert_eq!(
                Bitboard::single(Square::SQ_1A) | Bitboard::single(Square::SQ_1B),
                bb.decrement()
            );
        }
        {
            let bb = Bitboard::single(Square::SQ_9I);
            assert_eq!(
                !Bitboard::single(Square::SQ_9I),
                bb.decrement() & !Bitboard::empty()
            );
        }
    }
}
