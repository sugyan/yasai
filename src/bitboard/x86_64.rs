use super::Occupied;
use once_cell::sync::Lazy;
use shogi_core::Square;
use std::arch::x86_64;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Bitboard(x86_64::__m128i);

static SINGLES: Lazy<[Bitboard; Square::NUM]> = Lazy::new(|| {
    let mut bbs = [Bitboard::empty(); Square::NUM];
    for sq in Square::all() {
        bbs[sq.array_index()] = Bitboard::from_square(sq);
    }
    bbs
});

static MASKED_BBS: Lazy<[Bitboard; Square::NUM + 2]> = Lazy::new(|| {
    let mut bbs = [Bitboard::empty(); Square::NUM + 2];
    for (i, bb) in bbs.iter_mut().enumerate() {
        let value = (1_u128 << i) - 1;
        let inner = [value as i64, (value >> 64) as i64];
        *bb = Bitboard(unsafe { x86_64::_mm_set_epi64x(inner[1], inner[0]) })
    }
    bbs
});

impl Bitboard {
    #[inline(always)]
    pub fn empty() -> Self {
        Self(unsafe { x86_64::_mm_setzero_si128() })
    }
    #[inline(always)]
    pub fn single(square: Square) -> Self {
        SINGLES[square.array_index()]
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
        if self.is_empty() {
            None
        } else {
            let sq;
            unsafe {
                let mut i0 = x86_64::_mm_extract_epi64::<0>(self.0);
                self.0 = if i0 != 0 {
                    sq = Square::from_u8_unchecked(Self::pop_lsb(&mut i0) + 1);
                    x86_64::_mm_insert_epi64::<0>(self.0, i0)
                } else {
                    let mut i1 = x86_64::_mm_extract_epi64::<1>(self.0);
                    sq = Square::from_u8_unchecked(Self::pop_lsb(&mut i1) + 64);
                    x86_64::_mm_insert_epi64::<1>(self.0, i1)
                }
            }
            Some(sq)
        }
    }
    #[inline(always)]
    fn pop_lsb(n: &mut i64) -> u8 {
        let ret = n.trailing_zeros() as u8;
        *n = *n & (*n - 1);
        ret
    }
    fn from_square(sq: Square) -> Self {
        let index = sq.index() - 1;
        let inner = if index < 63 {
            [1 << index, 0]
        } else {
            [0, 1 << (index - 63)]
        };
        Self(unsafe { x86_64::_mm_set_epi64x(inner[1], inner[0]) })
    }
    // FIXME
    fn leading_zeros(&self) -> u32 {
        unsafe {
            let i1 = x86_64::_mm_extract_epi64::<1>(self.0);
            if i1 != 0 {
                i1.leading_zeros()
            } else {
                64 + x86_64::_mm_extract_epi64::<0>(self.0).leading_zeros()
            }
        }
    }
    // FIXME
    fn trailing_zeros(&self) -> u32 {
        unsafe {
            let i0 = x86_64::_mm_extract_epi64::<0>(self.0);
            if i0 != 0 {
                i0.trailing_zeros()
            } else {
                64 + x86_64::_mm_extract_epi64::<1>(self.0).trailing_zeros()
            }
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
        let tz = (*self & *mask | Self::single(Square::SQ_9I)).trailing_zeros();
        *mask & MASKED_BBS[tz as usize + 1]
    }
    fn sliding_negative(&self, mask: &Self) -> Self {
        let lz = (*self & *mask | Self::single(Square::SQ_1A)).leading_zeros();
        *mask & !MASKED_BBS[127 - lz as usize]
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
        todo!()
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
