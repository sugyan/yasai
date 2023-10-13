pub(crate) trait Occupied
where
    Self: Sized,
{
    /// Shift left (South)
    fn shl(&self) -> Self;
    /// Shift right (North)
    fn shr(&self) -> Self;
    /// Slide consecutively to the positive: that is South.
    fn sliding_positive_consecutive(&self, mask: &Self) -> Self;
    /// Slide consecutively to the negative: that is North.
    fn sliding_negative_consecutive(&self, mask: &Self) -> Self;
    /// Slide for 2 directions to the positive. Positive is further West, or further South if it's on the same file.
    fn sliding_positives(&self, masks: &[Self; 2]) -> Self;
    /// Slide for 2 directions to the negative. Negative is further East, or further North if it's on the same file.
    fn sliding_negatives(&self, masks: &[Self; 2]) -> Self;
    /// Vacant files
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
        // Imagine there's a bishop at 6E
        let bb = to_bb(vec![SQ_8C, SQ_8G]);
        assert_eq!(
            bb | to_bb(vec![SQ_7D, SQ_7F]),
            bb.sliding_positives(&[
                to_bb(vec![SQ_7D, SQ_8C, SQ_9B]),
                to_bb(vec![SQ_7F, SQ_8G, SQ_9H]),
            ])
        );

        // Imagine there's a rook at 6F
        let bb = to_bb(vec![SQ_6H, SQ_8F]);
        assert_eq!(
            bb | to_bb(vec![SQ_6G, SQ_7F]),
            bb.sliding_positives(&[
                to_bb(vec![SQ_6G, SQ_6H, SQ_6I]),
                to_bb(vec![SQ_7F, SQ_8F, SQ_9F]),
            ])
        );
    }

    #[test]
    fn sliding_negatives() {
        // Imagine there's a bishop at 4E
        let bb = to_bb(vec![SQ_2C, SQ_2G]);
        assert_eq!(
            bb | to_bb(vec![SQ_3D, SQ_3F]),
            bb.sliding_negatives(&[
                to_bb(vec![SQ_3D, SQ_2C, SQ_1B]),
                to_bb(vec![SQ_3F, SQ_2G, SQ_1H]),
            ])
        );
        // Imagine there's a rook at 4D
        let bb = to_bb(vec![SQ_2D, SQ_4B]);
        assert_eq!(
            bb | to_bb(vec![SQ_3D, SQ_4C]),
            bb.sliding_negatives(&[
                to_bb(vec![SQ_3D, SQ_2D, SQ_1D]),
                to_bb(vec![SQ_4C, SQ_4B, SQ_4A]),
            ])
        );
    }

    #[test]
    fn vacant_files() {
        assert_eq!(!Bitboard::empty(), Bitboard::empty().vacant_files());
        let all_files = to_bb(vec![
            SQ_1A, SQ_2B, SQ_3C, SQ_4D, SQ_5E, SQ_6F, SQ_7G, SQ_8H, SQ_9I,
        ])
        .vacant_files();
        assert_eq!(Bitboard::empty(), all_files);

        let odd_files = to_bb(vec![SQ_1A, SQ_3A, SQ_5A, SQ_7A, SQ_9A]).vacant_files();
        let odd_files2 = to_bb(vec![SQ_1I, SQ_3I, SQ_5I, SQ_7I, SQ_9I]).vacant_files();
        assert_eq!(odd_files, odd_files2);

        let even_files = to_bb(vec![SQ_2A, SQ_4A, SQ_6A, SQ_8A]).vacant_files();
        assert_eq!(Bitboard::empty(), odd_files & even_files);
        assert_eq!(!Bitboard::empty(), odd_files | even_files);
    }

    fn to_bb(squares: Vec<Square>) -> Bitboard {
        squares
            .iter()
            .fold(Bitboard::empty(), |acc, e| (acc | Bitboard::single(*e)))
    }
}
