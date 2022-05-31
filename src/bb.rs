use once_cell::sync::Lazy;
use shogi_core::{Bitboard, Square};

pub(crate) trait BitboardExt {
    fn values(&self) -> (u64, u64);
    fn shl(&self) -> Bitboard;
    fn shr(&self) -> Bitboard;
}

impl BitboardExt for Bitboard {
    fn values(&self) -> (u64, u64) {
        let mut ret = (0, 0);
        for sq in *self {
            if sq.index() < 64 {
                ret.0 |= 1 << (sq.index() - 1);
            } else {
                ret.1 |= 1 << (sq.index() - 64);
            }
        }
        ret
    }
    fn shl(&self) -> Bitboard {
        let mut ret = Bitboard::empty();
        for sq in *self {
            ret |= Bitboard::single(unsafe { Square::from_u8_unchecked(sq.index() + 1) })
        }
        ret
    }
    fn shr(&self) -> Bitboard {
        let mut ret = Bitboard::empty();
        for sq in *self {
            ret |= Bitboard::single(unsafe { Square::from_u8_unchecked(sq.index() - 1) })
        }
        ret
    }
}

#[rustfmt::skip]
pub(crate) static FILES: Lazy<[Bitboard; 10]> = Lazy::new(|| [
    Bitboard::empty(),
    Bitboard::single(Square::SQ_1A) | Bitboard::single(Square::SQ_1B) | Bitboard::single(Square::SQ_1C) | Bitboard::single(Square::SQ_1D) | Bitboard::single(Square::SQ_1E) | Bitboard::single(Square::SQ_1F) | Bitboard::single(Square::SQ_1G) | Bitboard::single(Square::SQ_1H) | Bitboard::single(Square::SQ_1I),
    Bitboard::single(Square::SQ_2A) | Bitboard::single(Square::SQ_2B) | Bitboard::single(Square::SQ_2C) | Bitboard::single(Square::SQ_2D) | Bitboard::single(Square::SQ_2E) | Bitboard::single(Square::SQ_2F) | Bitboard::single(Square::SQ_2G) | Bitboard::single(Square::SQ_2H) | Bitboard::single(Square::SQ_2I),
    Bitboard::single(Square::SQ_3A) | Bitboard::single(Square::SQ_3B) | Bitboard::single(Square::SQ_3C) | Bitboard::single(Square::SQ_3D) | Bitboard::single(Square::SQ_3E) | Bitboard::single(Square::SQ_3F) | Bitboard::single(Square::SQ_3G) | Bitboard::single(Square::SQ_3H) | Bitboard::single(Square::SQ_3I),
    Bitboard::single(Square::SQ_4A) | Bitboard::single(Square::SQ_4B) | Bitboard::single(Square::SQ_4C) | Bitboard::single(Square::SQ_4D) | Bitboard::single(Square::SQ_4E) | Bitboard::single(Square::SQ_4F) | Bitboard::single(Square::SQ_4G) | Bitboard::single(Square::SQ_4H) | Bitboard::single(Square::SQ_4I),
    Bitboard::single(Square::SQ_5A) | Bitboard::single(Square::SQ_5B) | Bitboard::single(Square::SQ_5C) | Bitboard::single(Square::SQ_5D) | Bitboard::single(Square::SQ_5E) | Bitboard::single(Square::SQ_5F) | Bitboard::single(Square::SQ_5G) | Bitboard::single(Square::SQ_5H) | Bitboard::single(Square::SQ_5I),
    Bitboard::single(Square::SQ_6A) | Bitboard::single(Square::SQ_6B) | Bitboard::single(Square::SQ_6C) | Bitboard::single(Square::SQ_6D) | Bitboard::single(Square::SQ_6E) | Bitboard::single(Square::SQ_6F) | Bitboard::single(Square::SQ_6G) | Bitboard::single(Square::SQ_6H) | Bitboard::single(Square::SQ_6I),
    Bitboard::single(Square::SQ_7A) | Bitboard::single(Square::SQ_7B) | Bitboard::single(Square::SQ_7C) | Bitboard::single(Square::SQ_7D) | Bitboard::single(Square::SQ_7E) | Bitboard::single(Square::SQ_7F) | Bitboard::single(Square::SQ_7G) | Bitboard::single(Square::SQ_7H) | Bitboard::single(Square::SQ_7I),
    Bitboard::single(Square::SQ_8A) | Bitboard::single(Square::SQ_8B) | Bitboard::single(Square::SQ_8C) | Bitboard::single(Square::SQ_8D) | Bitboard::single(Square::SQ_8E) | Bitboard::single(Square::SQ_8F) | Bitboard::single(Square::SQ_8G) | Bitboard::single(Square::SQ_8H) | Bitboard::single(Square::SQ_8I),
    Bitboard::single(Square::SQ_9A) | Bitboard::single(Square::SQ_9B) | Bitboard::single(Square::SQ_9C) | Bitboard::single(Square::SQ_9D) | Bitboard::single(Square::SQ_9E) | Bitboard::single(Square::SQ_9F) | Bitboard::single(Square::SQ_9G) | Bitboard::single(Square::SQ_9H) | Bitboard::single(Square::SQ_9I),
]);
#[rustfmt::skip]
pub(crate) static RANKS: Lazy<[Bitboard; 10]> = Lazy::new(|| [
    Bitboard::empty(),
    Bitboard::single(Square::SQ_1A) | Bitboard::single(Square::SQ_2A) | Bitboard::single(Square::SQ_3A) | Bitboard::single(Square::SQ_4A) | Bitboard::single(Square::SQ_5A) | Bitboard::single(Square::SQ_6A) | Bitboard::single(Square::SQ_7A) | Bitboard::single(Square::SQ_8A) | Bitboard::single(Square::SQ_9A),
    Bitboard::single(Square::SQ_1B) | Bitboard::single(Square::SQ_2B) | Bitboard::single(Square::SQ_3B) | Bitboard::single(Square::SQ_4B) | Bitboard::single(Square::SQ_5B) | Bitboard::single(Square::SQ_6B) | Bitboard::single(Square::SQ_7B) | Bitboard::single(Square::SQ_8B) | Bitboard::single(Square::SQ_9B),
    Bitboard::single(Square::SQ_1C) | Bitboard::single(Square::SQ_2C) | Bitboard::single(Square::SQ_3C) | Bitboard::single(Square::SQ_4C) | Bitboard::single(Square::SQ_5C) | Bitboard::single(Square::SQ_6C) | Bitboard::single(Square::SQ_7C) | Bitboard::single(Square::SQ_8C) | Bitboard::single(Square::SQ_9C),
    Bitboard::single(Square::SQ_1D) | Bitboard::single(Square::SQ_2D) | Bitboard::single(Square::SQ_3D) | Bitboard::single(Square::SQ_4D) | Bitboard::single(Square::SQ_5D) | Bitboard::single(Square::SQ_6D) | Bitboard::single(Square::SQ_7D) | Bitboard::single(Square::SQ_8D) | Bitboard::single(Square::SQ_9D),
    Bitboard::single(Square::SQ_1E) | Bitboard::single(Square::SQ_2E) | Bitboard::single(Square::SQ_3E) | Bitboard::single(Square::SQ_4E) | Bitboard::single(Square::SQ_5E) | Bitboard::single(Square::SQ_6E) | Bitboard::single(Square::SQ_7E) | Bitboard::single(Square::SQ_8E) | Bitboard::single(Square::SQ_9E),
    Bitboard::single(Square::SQ_1F) | Bitboard::single(Square::SQ_2F) | Bitboard::single(Square::SQ_3F) | Bitboard::single(Square::SQ_4F) | Bitboard::single(Square::SQ_5F) | Bitboard::single(Square::SQ_6F) | Bitboard::single(Square::SQ_7F) | Bitboard::single(Square::SQ_8F) | Bitboard::single(Square::SQ_9F),
    Bitboard::single(Square::SQ_1G) | Bitboard::single(Square::SQ_2G) | Bitboard::single(Square::SQ_3G) | Bitboard::single(Square::SQ_4G) | Bitboard::single(Square::SQ_5G) | Bitboard::single(Square::SQ_6G) | Bitboard::single(Square::SQ_7G) | Bitboard::single(Square::SQ_8G) | Bitboard::single(Square::SQ_9G),
    Bitboard::single(Square::SQ_1H) | Bitboard::single(Square::SQ_2H) | Bitboard::single(Square::SQ_3H) | Bitboard::single(Square::SQ_4H) | Bitboard::single(Square::SQ_5H) | Bitboard::single(Square::SQ_6H) | Bitboard::single(Square::SQ_7H) | Bitboard::single(Square::SQ_8H) | Bitboard::single(Square::SQ_9H),
    Bitboard::single(Square::SQ_1I) | Bitboard::single(Square::SQ_2I) | Bitboard::single(Square::SQ_3I) | Bitboard::single(Square::SQ_4I) | Bitboard::single(Square::SQ_5I) | Bitboard::single(Square::SQ_6I) | Bitboard::single(Square::SQ_7I) | Bitboard::single(Square::SQ_8I) | Bitboard::single(Square::SQ_9I),
]);
