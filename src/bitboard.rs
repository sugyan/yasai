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
