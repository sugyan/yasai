use super::Occupied;
pub(crate) use shogi_core::Bitboard;
use shogi_core::Square;

const VACANT_MASK_VALUE: u128 = 0x0002_0100_4020_1008_0402_0100;
const VACANT_MASK: Bitboard = unsafe { Bitboard::from_u128_unchecked(VACANT_MASK_VALUE) };
const BB_1A: Bitboard = Bitboard::single(Square::SQ_1A);
const BB_9I: Bitboard = Bitboard::single(Square::SQ_9I);

const MASKED_BBS: [Bitboard; Square::NUM + 2] = [
    Bitboard::empty(),
    unsafe { Bitboard::from_u128_unchecked((1 << 1) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 2) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 3) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 4) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 5) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 6) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 7) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 8) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 9) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 10) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 11) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 12) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 13) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 14) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 15) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 16) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 17) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 18) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 19) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 20) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 21) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 22) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 23) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 24) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 25) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 26) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 27) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 28) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 29) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 30) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 31) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 32) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 33) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 34) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 35) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 36) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 37) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 38) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 39) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 40) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 41) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 42) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 43) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 44) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 45) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 46) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 47) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 48) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 49) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 50) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 51) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 52) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 53) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 54) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 55) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 56) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 57) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 58) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 59) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 60) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 61) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 62) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 63) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 64) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 65) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 66) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 67) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 68) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 69) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 70) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 71) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 72) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 73) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 74) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 75) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 76) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 77) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 78) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 79) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 80) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 81) - 1) },
    unsafe { Bitboard::from_u128_unchecked((1 << 82) - 1) },
];

#[inline(always)]
fn sliding_positive(bb: &Bitboard, mask: &Bitboard) -> Bitboard {
    let tz = (*bb & *mask | BB_9I).to_u128().trailing_zeros();
    *mask & MASKED_BBS[tz as usize + 1]
}

#[inline(always)]
fn sliding_negative(bb: &Bitboard, mask: &Bitboard) -> Bitboard {
    let lz = (*bb & *mask | BB_1A).to_u128().leading_zeros();
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
        let bb = unsafe { Self::from_u128_unchecked(VACANT_MASK_VALUE - self.to_u128()) };
        VACANT_MASK
            ^ unsafe { Self::from_u128_unchecked(VACANT_MASK_VALUE - bb.shift_up(8).to_u128()) }
    }
}
