use super::Occupied;
pub(crate) use shogi_core::Bitboard;
use shogi_core::Square;

//                                                            0001 _ 1111 1110 1111 1111
// 0011 1111 1101 1111 _ 1110 1111 1111 0111 _ 1111 1011 1111 1101 _ 1111 1110 1111 1111
const FILL_MASK_VALUE: u128 = 0x0001_feff_3fdf_eff7_fbfd_feff;
const BB_1A: Bitboard = Bitboard::single(Square::SQ_1A);
const BB_9I: Bitboard = Bitboard::single(Square::SQ_9I);

impl Occupied for Bitboard {
    fn sliding_positive(&self, mask: &Self) -> Self {
        let tz = (*self & *mask | BB_9I).to_u128().trailing_zeros();
        *mask & unsafe { Self::from_u128_unchecked((1 << (tz + 1)) - 1) }
    }
    fn sliding_negative(&self, mask: &Self) -> Self {
        let lz = (*self & *mask | BB_1A).to_u128().leading_zeros();
        *mask & unsafe { Self::from_u128_unchecked(!((1 << (127 - lz)) - 1)) }
    }
    fn filled_files(&self) -> Self {
        let bb = unsafe { Self::from_u128_unchecked(self.to_u128() + FILL_MASK_VALUE + (1 << 63)) };
        (unsafe { Self::from_u128_unchecked(FILL_MASK_VALUE) })
            ^ unsafe { Self::from_u128_unchecked(bb.shift_up(8).to_u128() + FILL_MASK_VALUE) }
    }
}
