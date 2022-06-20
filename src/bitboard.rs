pub(crate) trait Occupied
where
    Self: Sized,
{
    fn shl(&self, rhs: u8) -> Self;
    fn shr(&self, rhs: u8) -> Self;
    fn sliding_positive(&self, mask: &Self) -> Self;
    fn sliding_negative(&self, mask: &Self) -> Self;
    fn sliding_positives(&self, masks: &[Self; 2]) -> Self;
    fn sliding_negatives(&self, masks: &[Self; 2]) -> Self;
    fn vacant_files(&self) -> Self;
}

cfg_if::cfg_if! {
    if #[cfg(all(
        feature = "simd",
        target_arch = "x86_64",
        target_feature = "sse4.1"
    ))] {
        mod x86_64;
        pub(crate) use self::x86_64::Bitboard;
    } else {
        mod core;
        pub(crate) use self::core::Bitboard;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shogi_core::Square;

    #[test]
    fn empty() {
        let bb = Bitboard::empty();
        assert!(bb.is_empty());
    }

    #[test]
    fn bit_ops() {
        let bb0 = Bitboard::empty();
        let bb1 = Bitboard::single(Square::SQ_1A);
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
}
