use once_cell::sync::Lazy;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use shogi_core::{Color, Hand, Piece, PieceKind, Square};
use std::ops;

#[derive(Clone, Copy, Debug)]
pub struct Key(u64);

impl Key {
    pub const ZERO: Key = Key(0);
    pub const COLOR: Key = Key(1);

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl ops::Not for Key {
    type Output = Self;

    fn not(self) -> Self::Output {
        Key(!self.0)
    }
}

impl ops::BitAnd for Key {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Key(self.0 & rhs.0)
    }
}

impl ops::BitXor for Key {
    type Output = Self;

    fn bitxor(self, rhs: Key) -> Self::Output {
        Key(self.0 ^ rhs.0)
    }
}

impl ops::BitXorAssign for Key {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

pub struct ZobristTable {
    board: [[[Key; PieceKind::NUM]; Color::NUM]; Square::NUM],
    hands: [[[Key; ZobristTable::MAX_HAND_NUM]; 8]; Color::NUM],
}

impl ZobristTable {
    const MAX_HAND_NUM: usize = 18;

    pub fn board(&self, sq: Square, p: Piece) -> Key {
        self.board[sq.array_index()][p.color().array_index()][p.piece_kind().array_index()]
    }
    pub fn hand(&self, c: Color, pk: PieceKind, num: u8) -> Key {
        self.hands[c.array_index()][pk.array_index()][num as usize]
    }
}

pub static ZOBRIST_TABLE: Lazy<ZobristTable> = Lazy::new(|| {
    let mut board = [[[Key::ZERO; PieceKind::NUM]; Color::NUM]; Square::NUM];
    let mut hands = [[[Key::ZERO; ZobristTable::MAX_HAND_NUM]; 8]; Color::NUM];
    let mut rng = StdRng::seed_from_u64(2022);
    for sq in Square::all() {
        for c in Color::all() {
            for pk in PieceKind::all() {
                board[sq.array_index()][c.array_index()][pk.array_index()] =
                    Key(rng.gen()) & !Key::COLOR;
            }
        }
    }
    let h = Hand::new();
    for c in Color::all() {
        for pk in PieceKind::all() {
            if h.count(pk).is_none() {
                continue;
            }
            for num in 0..ZobristTable::MAX_HAND_NUM {
                hands[c.array_index()][pk.array_index()][num] = Key(rng.gen()) & !Key::COLOR;
            }
        }
    }
    ZobristTable { board, hands }
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Move, Position};
    use std::collections::HashSet;

    #[test]
    fn empty() {
        let pos = Position::new([None; Square::NUM], [[0; 8]; 2], Color::Black, 1);
        assert_eq!(0, pos.key());
    }

    #[test]
    fn default() {
        let pos = Position::default();
        assert_ne!(0, pos.key());
    }

    #[test]
    fn full_hands() {
        let pos = Position::new(
            [None; Square::NUM],
            [[18, 4, 4, 4, 4, 2, 2, 2], [0; 8]],
            Color::Black,
            1,
        );
        assert_ne!(0, pos.key());
    }

    #[test]
    fn uniqueness() {
        let mut hs = HashSet::new();
        let mut pos = Position::default();
        for i in 0..100 {
            let moves = pos.legal_moves().into_iter().collect::<Vec<_>>();
            let choice = moves[(i * 100) % moves.len()];
            pos.do_move(choice);
            let key = pos.key();
            assert_eq!(key % 2 == 0, i % 2 == 1);
            hs.insert(key);
        }
        assert_eq!(100, hs.len());
    }

    #[test]
    fn joined() {
        // P1-KY-KE-GI-KI-OU-KI-GI-KE-KY
        // P2 * -HI *  *  *  *  * -KA *
        // P3-FU-FU-FU-FU-FU-FU * -FU-FU
        // P4 *  *  *  *  *  * -FU *  *
        // P5 *  *  *  *  *  *  *  *  *
        // P6 *  * +FU *  *  *  * +FU *
        // P7+FU+FU * +FU+FU+FU+FU * +FU
        // P8 *  *  *  *  *  *  * +HI *
        // P9+KY+KE+GI+KI+OU+KI+GI+KE+KY
        // P+00KA
        // P-00KA
        // -
        let key0 = {
            let mut pos = Position::default();
            // +7776FU,-3334FU,+2726FU
            let moves = [
                Move::new_normal(Square::SQ_7G, Square::SQ_7F, false, Piece::B_P),
                Move::new_normal(Square::SQ_3C, Square::SQ_3D, false, Piece::W_P),
                Move::new_normal(Square::SQ_2G, Square::SQ_2F, false, Piece::B_P),
            ];
            moves.iter().for_each(|&m| pos.do_move(m));
            pos.key()
        };
        let key1 = {
            let mut pos = Position::default();
            // +2726FU,-3334FU,+7776FU
            let moves = [
                Move::new_normal(Square::SQ_2G, Square::SQ_2F, false, Piece::B_P),
                Move::new_normal(Square::SQ_7G, Square::SQ_7F, false, Piece::B_P),
                Move::new_normal(Square::SQ_3C, Square::SQ_3D, false, Piece::W_P),
            ];
            moves.iter().for_each(|&m| pos.do_move(m));
            pos.key()
        };
        assert_eq!(key0, key1);
    }

    #[test]
    fn same_board() {
        // P1-KY-KE-GI-KI-OU-KI-GI-KE-KY
        // P2 * -HI *  *  *  *  *  *  *
        // P3-FU-FU-FU-FU-FU-FU * -FU-FU
        // P4 *  *  *  *  *  * -FU *  *
        // P5 *  *  *  *  *  *  *  *  *
        // P6 *  * +FU *  *  *  *  *  *
        // P7+FU+FU * +FU+FU+FU+FU+FU+FU
        // P8 * +KA *  *  *  *  * +HI *
        // P9+KY+KE+GI+KI+OU+KI+GI+KE+KY
        // +
        let keys0 = {
            let mut pos = Position::default();
            // +7776FU,-3334FU,+8822KA,-3122GI,+0088KA,-2231GI
            // => P-00KA
            let moves = [
                Move::new_normal(Square::SQ_7G, Square::SQ_7F, false, Piece::B_P),
                Move::new_normal(Square::SQ_3C, Square::SQ_3D, false, Piece::W_P),
                Move::new_normal(Square::SQ_8H, Square::SQ_2B, false, Piece::B_B),
                Move::new_normal(Square::SQ_3A, Square::SQ_2B, false, Piece::W_S),
                Move::new_drop(Square::SQ_8H, Piece::B_B),
                Move::new_normal(Square::SQ_2B, Square::SQ_3A, false, Piece::W_S),
            ];
            moves.iter().for_each(|&m| pos.do_move(m));
            pos.keys()
        };
        let keys1 = {
            let mut pos = Position::default();
            // +7776FU,-3334FU,+8822KA,-3142GI,+2288KA,-4231GI
            // => P+00KA
            let moves = [
                Move::new_normal(Square::SQ_7G, Square::SQ_7F, false, Piece::B_P),
                Move::new_normal(Square::SQ_3C, Square::SQ_3D, false, Piece::W_P),
                Move::new_normal(Square::SQ_8H, Square::SQ_2B, false, Piece::B_B),
                Move::new_normal(Square::SQ_3A, Square::SQ_4B, false, Piece::W_S),
                Move::new_normal(Square::SQ_2B, Square::SQ_8H, false, Piece::B_B),
                Move::new_normal(Square::SQ_4B, Square::SQ_3A, false, Piece::W_S),
            ];
            moves.iter().for_each(|&m| pos.do_move(m));
            pos.keys()
        };
        assert_ne!(keys0, keys1);
        assert_eq!(keys0.0, keys1.0)
    }
}