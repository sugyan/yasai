pub(crate) trait Occupied
where
    Self: Sized,
{
    fn shl(&self) -> Self;
    fn shr(&self) -> Self;
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
        assert!(Bitboard::empty().is_empty());
    }

    #[test]
    fn contains() {
        for sq in Square::all() {
            let bb = Bitboard::single(sq);
            assert!(bb.contains(sq));
        }
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

    #[test]
    fn pop() {
        assert_eq!(None, Bitboard::empty().pop());
        for sq in Square::all() {
            let mut bb = Bitboard::single(sq);
            assert_eq!(Some(sq), bb.pop());
            assert!(bb.is_empty());
            assert_eq!(None, bb.pop());
        }
        assert_eq!(Vec::<Square>::new(), Bitboard::empty().collect::<Vec<_>>());
        assert_eq!(
            Square::all().collect::<Vec<_>>(),
            (!Bitboard::empty()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn shift() {
        assert_eq!(
            Bitboard::single(Square::SQ_1B),
            Bitboard::single(Square::SQ_1A).shl()
        );
        assert_eq!(
            Bitboard::single(Square::SQ_9H),
            Bitboard::single(Square::SQ_9I).shr()
        );
    }
}
