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
    pub fn contains(&self, square: Square) -> bool {
        todo!()
    }
    pub const unsafe fn shift_down(&self, delta: u8) -> Self {
        todo!()
    }
    pub const unsafe fn shift_up(&self, delta: u8) -> Self {
        todo!()
    }
    pub fn pop(&mut self) -> Option<Square> {
        todo!()
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
