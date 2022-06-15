pub(crate) trait Occupied {
    fn sliding_positive(&self, mask: &Self) -> Self;
    fn sliding_negative(&self, mask: &Self) -> Self;
    fn filled_files(&self) -> Self;
}

mod core;
pub(crate) use self::core::Bitboard;
