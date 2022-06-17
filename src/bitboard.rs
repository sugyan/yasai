pub(crate) trait Occupied
where
    Self: Sized,
{
    fn sliding_positive(&self, mask: &Self) -> Self;
    fn sliding_negative(&self, mask: &Self) -> Self;
    fn sliding_positives(&self, masks: &[Self; 2]) -> Self;
    fn sliding_negatives(&self, masks: &[Self; 2]) -> Self;
    fn vacant_files(&self) -> Self;
}

mod core;
pub(crate) use self::core::Bitboard;
