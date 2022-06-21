use super::Occupied;
pub(crate) use shogi_core::Bitboard;
use shogi_core::Square;

const VACANT_MASK_VALUE: u128 = 0x0002_0100_4020_1008_0402_0100;
const VACANT_MASK: Bitboard = unsafe { Bitboard::from_u128_unchecked(VACANT_MASK_VALUE) };
const BB_1A: Bitboard = Bitboard::single(Square::SQ_1A);
const BB_9I: Bitboard = Bitboard::single(Square::SQ_9I);

const MASKED_BBS: [Bitboard; Square::NUM + 2] = {
    let mask = 0x0003_ffff_7fff_ffff_ffff_ffff;
    let mut bbs = [Bitboard::empty(); Square::NUM + 2];
    let mut i = 0;
    while i < Square::NUM + 2 {
        bbs[i] = unsafe { Bitboard::from_u128_unchecked(mask & ((1 << i) - 1)) };
        i += 1;
    }
    bbs
};

impl Occupied for Bitboard {
    #[inline(always)]
    fn sliding_positive(&self, mask: &Self) -> Self {
        let tz = (*self & *mask | BB_9I).to_u128().trailing_zeros();
        *mask & MASKED_BBS[tz as usize + 1]
    }
    #[inline(always)]
    fn sliding_negative(&self, mask: &Self) -> Self {
        let lz = (*self & *mask | BB_1A).to_u128().leading_zeros();
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
    #[inline(always)]
    fn vacant_files(&self) -> Self {
        let bb = unsafe { Self::from_u128_unchecked(VACANT_MASK_VALUE - self.to_u128()) };
        VACANT_MASK
            ^ unsafe { Self::from_u128_unchecked(VACANT_MASK_VALUE - bb.shift_up(8).to_u128()) }
    }
}
