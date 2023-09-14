use super::Occupied;
pub(crate) use shogi_core::Bitboard;
use shogi_core::Square;

/// Note the alignment of the bitboard: 18 bits and 63 bits out of 2 64-bit int are used
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

/// # Arguments
///
/// * `bb` - The occupied bitboard
/// * `mask` - The potential attacks
#[inline(always)]
fn sliding_positive(bb: &Bitboard, mask: &Bitboard) -> Bitboard {
    let tz = (*bb & mask | BB_9I).to_u128().trailing_zeros();
    *mask & MASKED_BBS[tz as usize + 1]
}

/// # Arguments
///
/// * `bb` - The occupied bitboard
/// * `mask` - The potential attacks
#[inline(always)]
fn sliding_negative(bb: &Bitboard, mask: &Bitboard) -> Bitboard {
    let lz = (*bb & mask | BB_1A).to_u128().leading_zeros();
    *mask & !MASKED_BBS[127 - lz as usize]
}

impl Occupied for Bitboard {
    #[inline(always)]
    fn shl(&self) -> Self {
        unsafe { self.shift_down(1) }
    }
    #[inline(always)]
    fn shr(&self) -> Self {
        unsafe { self.shift_up(1) }
    }
    #[inline(always)]
    fn sliding_positive_consecutive(&self, mask: &Self) -> Self {
        sliding_positive(self, mask)
    }
    #[inline(always)]
    fn sliding_negative_consecutive(&self, mask: &Self) -> Self {
        sliding_negative(self, mask)
    }
    #[inline(always)]
    fn sliding_positives(&self, masks: &[Self; 2]) -> Self {
        sliding_positive(self, &masks[0]) | sliding_positive(self, &masks[1])
    }
    #[inline(always)]
    fn sliding_negatives(&self, masks: &[Self; 2]) -> Self {
        sliding_negative(self, &masks[0]) | sliding_negative(self, &masks[1])
    }
    #[inline(always)]
    fn vacant_files(&self) -> Self {
        // Following happens in parallel for each file:
        // 1. The highest bit of (0b100000000 - self) is 1 iff the file is vacant thanks to borrowing.
        // 2. Shift it by 8 bit to get the flag. Results in either 0b000000000 or 0b000000001
        // 3. 0b100000000 - the value from 2. Results in either 0b100000000 or 0b011111111
        // 4. XOR with 0b100000000. Results in either 0b000000000 or 0b111111111
        let bb = unsafe { Self::from_u128_unchecked(VACANT_MASK_VALUE - self.to_u128()) };
        VACANT_MASK
            ^ unsafe { Self::from_u128_unchecked(VACANT_MASK_VALUE - bb.shift_up(8).to_u128()) }
    }
}
