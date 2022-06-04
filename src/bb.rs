use shogi_core::{Bitboard, Square};

pub(crate) trait BitboardExt {
    fn values(&self) -> (u64, u64);
}

impl BitboardExt for Bitboard {
    fn values(&self) -> (u64, u64) {
        let repr = self.to_u128();
        ((repr & 0x7fff_ffff_ffff_ffff) as u64, (repr >> 64) as u64)
    }
}

#[rustfmt::skip]
pub(crate) const FILES: [Bitboard; 10] = [
    Bitboard::empty(),
    Bitboard::single(Square::SQ_1A).or(Bitboard::single(Square::SQ_1B)).or(Bitboard::single(Square::SQ_1C)).or(Bitboard::single(Square::SQ_1D)).or(Bitboard::single(Square::SQ_1E)).or(Bitboard::single(Square::SQ_1F)).or(Bitboard::single(Square::SQ_1G)).or(Bitboard::single(Square::SQ_1H)).or(Bitboard::single(Square::SQ_1I)),
    Bitboard::single(Square::SQ_2A).or(Bitboard::single(Square::SQ_2B)).or(Bitboard::single(Square::SQ_2C)).or(Bitboard::single(Square::SQ_2D)).or(Bitboard::single(Square::SQ_2E)).or(Bitboard::single(Square::SQ_2F)).or(Bitboard::single(Square::SQ_2G)).or(Bitboard::single(Square::SQ_2H)).or(Bitboard::single(Square::SQ_2I)),
    Bitboard::single(Square::SQ_3A).or(Bitboard::single(Square::SQ_3B)).or(Bitboard::single(Square::SQ_3C)).or(Bitboard::single(Square::SQ_3D)).or(Bitboard::single(Square::SQ_3E)).or(Bitboard::single(Square::SQ_3F)).or(Bitboard::single(Square::SQ_3G)).or(Bitboard::single(Square::SQ_3H)).or(Bitboard::single(Square::SQ_3I)),
    Bitboard::single(Square::SQ_4A).or(Bitboard::single(Square::SQ_4B)).or(Bitboard::single(Square::SQ_4C)).or(Bitboard::single(Square::SQ_4D)).or(Bitboard::single(Square::SQ_4E)).or(Bitboard::single(Square::SQ_4F)).or(Bitboard::single(Square::SQ_4G)).or(Bitboard::single(Square::SQ_4H)).or(Bitboard::single(Square::SQ_4I)),
    Bitboard::single(Square::SQ_5A).or(Bitboard::single(Square::SQ_5B)).or(Bitboard::single(Square::SQ_5C)).or(Bitboard::single(Square::SQ_5D)).or(Bitboard::single(Square::SQ_5E)).or(Bitboard::single(Square::SQ_5F)).or(Bitboard::single(Square::SQ_5G)).or(Bitboard::single(Square::SQ_5H)).or(Bitboard::single(Square::SQ_5I)),
    Bitboard::single(Square::SQ_6A).or(Bitboard::single(Square::SQ_6B)).or(Bitboard::single(Square::SQ_6C)).or(Bitboard::single(Square::SQ_6D)).or(Bitboard::single(Square::SQ_6E)).or(Bitboard::single(Square::SQ_6F)).or(Bitboard::single(Square::SQ_6G)).or(Bitboard::single(Square::SQ_6H)).or(Bitboard::single(Square::SQ_6I)),
    Bitboard::single(Square::SQ_7A).or(Bitboard::single(Square::SQ_7B)).or(Bitboard::single(Square::SQ_7C)).or(Bitboard::single(Square::SQ_7D)).or(Bitboard::single(Square::SQ_7E)).or(Bitboard::single(Square::SQ_7F)).or(Bitboard::single(Square::SQ_7G)).or(Bitboard::single(Square::SQ_7H)).or(Bitboard::single(Square::SQ_7I)),
    Bitboard::single(Square::SQ_8A).or(Bitboard::single(Square::SQ_8B)).or(Bitboard::single(Square::SQ_8C)).or(Bitboard::single(Square::SQ_8D)).or(Bitboard::single(Square::SQ_8E)).or(Bitboard::single(Square::SQ_8F)).or(Bitboard::single(Square::SQ_8G)).or(Bitboard::single(Square::SQ_8H)).or(Bitboard::single(Square::SQ_8I)),
    Bitboard::single(Square::SQ_9A).or(Bitboard::single(Square::SQ_9B)).or(Bitboard::single(Square::SQ_9C)).or(Bitboard::single(Square::SQ_9D)).or(Bitboard::single(Square::SQ_9E)).or(Bitboard::single(Square::SQ_9F)).or(Bitboard::single(Square::SQ_9G)).or(Bitboard::single(Square::SQ_9H)).or(Bitboard::single(Square::SQ_9I)),
];

#[rustfmt::skip]
pub(crate) const RANKS: [Bitboard; 10] = [
    Bitboard::empty(),
    Bitboard::single(Square::SQ_1A).or(Bitboard::single(Square::SQ_2A)).or(Bitboard::single(Square::SQ_3A)).or(Bitboard::single(Square::SQ_4A)).or(Bitboard::single(Square::SQ_5A)).or(Bitboard::single(Square::SQ_6A)).or(Bitboard::single(Square::SQ_7A)).or(Bitboard::single(Square::SQ_8A)).or(Bitboard::single(Square::SQ_9A)),
    Bitboard::single(Square::SQ_1B).or(Bitboard::single(Square::SQ_2B)).or(Bitboard::single(Square::SQ_3B)).or(Bitboard::single(Square::SQ_4B)).or(Bitboard::single(Square::SQ_5B)).or(Bitboard::single(Square::SQ_6B)).or(Bitboard::single(Square::SQ_7B)).or(Bitboard::single(Square::SQ_8B)).or(Bitboard::single(Square::SQ_9B)),
    Bitboard::single(Square::SQ_1C).or(Bitboard::single(Square::SQ_2C)).or(Bitboard::single(Square::SQ_3C)).or(Bitboard::single(Square::SQ_4C)).or(Bitboard::single(Square::SQ_5C)).or(Bitboard::single(Square::SQ_6C)).or(Bitboard::single(Square::SQ_7C)).or(Bitboard::single(Square::SQ_8C)).or(Bitboard::single(Square::SQ_9C)),
    Bitboard::single(Square::SQ_1D).or(Bitboard::single(Square::SQ_2D)).or(Bitboard::single(Square::SQ_3D)).or(Bitboard::single(Square::SQ_4D)).or(Bitboard::single(Square::SQ_5D)).or(Bitboard::single(Square::SQ_6D)).or(Bitboard::single(Square::SQ_7D)).or(Bitboard::single(Square::SQ_8D)).or(Bitboard::single(Square::SQ_9D)),
    Bitboard::single(Square::SQ_1E).or(Bitboard::single(Square::SQ_2E)).or(Bitboard::single(Square::SQ_3E)).or(Bitboard::single(Square::SQ_4E)).or(Bitboard::single(Square::SQ_5E)).or(Bitboard::single(Square::SQ_6E)).or(Bitboard::single(Square::SQ_7E)).or(Bitboard::single(Square::SQ_8E)).or(Bitboard::single(Square::SQ_9E)),
    Bitboard::single(Square::SQ_1F).or(Bitboard::single(Square::SQ_2F)).or(Bitboard::single(Square::SQ_3F)).or(Bitboard::single(Square::SQ_4F)).or(Bitboard::single(Square::SQ_5F)).or(Bitboard::single(Square::SQ_6F)).or(Bitboard::single(Square::SQ_7F)).or(Bitboard::single(Square::SQ_8F)).or(Bitboard::single(Square::SQ_9F)),
    Bitboard::single(Square::SQ_1G).or(Bitboard::single(Square::SQ_2G)).or(Bitboard::single(Square::SQ_3G)).or(Bitboard::single(Square::SQ_4G)).or(Bitboard::single(Square::SQ_5G)).or(Bitboard::single(Square::SQ_6G)).or(Bitboard::single(Square::SQ_7G)).or(Bitboard::single(Square::SQ_8G)).or(Bitboard::single(Square::SQ_9G)),
    Bitboard::single(Square::SQ_1H).or(Bitboard::single(Square::SQ_2H)).or(Bitboard::single(Square::SQ_3H)).or(Bitboard::single(Square::SQ_4H)).or(Bitboard::single(Square::SQ_5H)).or(Bitboard::single(Square::SQ_6H)).or(Bitboard::single(Square::SQ_7H)).or(Bitboard::single(Square::SQ_8H)).or(Bitboard::single(Square::SQ_9H)),
    Bitboard::single(Square::SQ_1I).or(Bitboard::single(Square::SQ_2I)).or(Bitboard::single(Square::SQ_3I)).or(Bitboard::single(Square::SQ_4I)).or(Bitboard::single(Square::SQ_5I)).or(Bitboard::single(Square::SQ_6I)).or(Bitboard::single(Square::SQ_7I)).or(Bitboard::single(Square::SQ_8I)).or(Bitboard::single(Square::SQ_9I)),
];
