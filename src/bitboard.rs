pub(crate) trait Occupied
where
    Self: Sized,
{
    fn shl(&self) -> Self;
    fn shr(&self) -> Self;
    fn sliding_positive_consecutive(&self, mask: &Self) -> Self;
    fn sliding_negative_consecutive(&self, mask: &Self) -> Self;
    fn sliding_positives(&self, masks: &[Self; 2]) -> Self;
    fn sliding_negatives(&self, masks: &[Self; 2]) -> Self;
    fn vacant_files(&self) -> Self;
}

#[allow(unused_macros)]
macro_rules! define_bit_trait {
    (
        target_trait => $trait:ident, assign_trait => $assign_trait:ident,
        target_func  => $func:ident,  assign_func  => $assign_func:ident,
        intrinsic    => $intrinsic:path
    ) => {
        impl $trait for Bitboard {
            type Output = Bitboard;

            #[inline(always)]
            fn $func(self, rhs: Self) -> Self::Output {
                Self($intrinsic(self.0, rhs.0))
            }
        }
        impl $trait<&Bitboard> for Bitboard {
            type Output = Bitboard;

            #[inline(always)]
            fn $func(self, rhs: &Self) -> Self::Output {
                Self($intrinsic(self.0, rhs.0))
            }
        }
        impl $assign_trait for Bitboard {
            #[inline(always)]
            fn $assign_func(&mut self, rhs: Self) {
                self.0 = $intrinsic(self.0, rhs.0)
            }
        }
    };
}

#[allow(unused_macros)]
macro_rules! define_bit_trait_unsafe {
    (
        target_trait => $trait:ident, assign_trait => $assign_trait:ident,
        target_func  => $func:ident,  assign_func  => $assign_func:ident,
        intrinsic    => $intrinsic:path
    ) => {
        impl $trait for Bitboard {
            type Output = Bitboard;

            #[inline(always)]
            fn $func(self, rhs: Self) -> Self::Output {
                Self(unsafe { $intrinsic(self.0, rhs.0) })
            }
        }
        impl $trait<&Bitboard> for Bitboard {
            type Output = Bitboard;

            #[inline(always)]
            fn $func(self, rhs: &Self) -> Self::Output {
                Self(unsafe { $intrinsic(self.0, rhs.0) })
            }
        }
        impl $assign_trait for Bitboard {
            #[inline(always)]
            fn $assign_func(&mut self, rhs: Self) {
                self.0 = unsafe { $intrinsic(self.0, rhs.0) }
            }
        }
    };
}

cfg_if::cfg_if! {
    if #[cfg(all(
        feature = "simd",
        target_arch = "x86_64",
        target_feature = "avx2"
    ))] {
        mod x86_64;
        pub(crate) use self::x86_64::Bitboard;
    } else if #[cfg(all(
        feature = "simd",
        target_arch = "aarch64",
        target_feature = "neon"
    ))] {
        mod aarch64;
        pub(crate) use self::aarch64::Bitboard;
    } else if #[cfg(all(
        feature = "simd",
        target_arch = "wasm32",
        target_feature = "simd128"
    ))] {
        mod wasm32;
        pub(crate) use self::wasm32::Bitboard;
    } else {
        mod core;
        pub(crate) use self::core::Bitboard;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shogi_core::consts::square::*;
    use shogi_core::Square;

    #[test]
    fn empty() {
        assert!(Bitboard::empty().is_empty());
    }

    #[test]
    fn single() {
        for sq in Square::all() {
            assert_eq!(1, Bitboard::single(sq).count());
        }
    }

    #[test]
    fn contains() {
        for sq in Square::all() {
            assert!(Bitboard::single(sq).contains(sq));
        }
    }

    #[test]
    fn bit_ops() {
        let bb0 = Bitboard::empty();
        let bb1 = Bitboard::single(SQ_1A);
        assert_eq!(bb0, bb0 & bb1);
        assert_eq!(bb1, bb0 | bb1);
        assert_eq!(bb1, bb0 ^ bb1);

        let mut bb = Bitboard::empty();
        assert_eq!(bb0, bb);
        bb |= bb1;
        assert_eq!(bb1, bb);
        bb &= bb1;
        assert_eq!(bb1, bb);
        bb ^= bb1;
        assert_eq!(bb0, bb);
    }

    #[test]
    fn shift() {
        assert_eq!(Bitboard::single(SQ_1B), Bitboard::single(SQ_1A).shl());
        assert_eq!(Bitboard::single(SQ_9H), Bitboard::single(SQ_9I).shr());
    }

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
