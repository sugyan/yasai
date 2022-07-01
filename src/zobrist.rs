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
    for c in Color::all() {
        for pk in Hand::all_hand_pieces() {
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
    use crate::Position;
    use shogi_core::{Move, PartialPosition};
    use shogi_usi_parser::FromUsi;
    use std::collections::HashSet;

    #[test]
    fn empty() {
        let pos = Position::new(PartialPosition::empty());
        assert_eq!(0, pos.key());
    }

    #[test]
    fn default() {
        let pos = Position::default();
        assert_ne!(0, pos.key());
    }

    #[test]
    fn full_hands() {
        let all_black_hands = Position::new(
            PartialPosition::from_usi("sfen 9/9/9/9/9/9/9/9/9 b 2R2B4G4S4N4L18P 1")
                .expect("failed to parse"),
        );
        let all_white_hands = Position::new(
            PartialPosition::from_usi("sfen 9/9/9/9/9/9/9/9/9 b 2r2b4g4s4n4l18p 1")
                .expect("failed to parse"),
        );
        assert_eq!(0, all_black_hands.keys().0);
        assert_eq!(0, all_white_hands.keys().0);
        assert_ne!(all_black_hands.keys().1, all_white_hands.keys().1);
    }

    #[test]
    fn capture_all_pawns() {
        // P1 *  *  *  * -OU *  *  * +OU
        // P2 *  *  *  *  *  *  * -TO *
        // P3 *  *  *  *  *  *  *  * -TO
        // P4 * -FU *  *  *  *  * -TO *
        // P5-FU * -FU *  *  *  *  * -TO
        // P6 * -TO * -FU *  *  * -TO *
        // P7-TO *  *  * -FU *  *  * -TO
        // P8 * -TO *  *  * -FU * -TO *
        // P9-TO *  *  *  *  * -TO *  *
        let mut pos = Position::new(
            PartialPosition::from_usi(
                "sfen 4k3K/7+p1/8+p/1p5+p1/p1p5+p/1+p1p3+p1/+p3p3+p/1+p3p1+p1/+p5+p2 b 2R2B4G4S4N4L 1",
            )
            .expect("failed to parse"),
        );
        #[rustfmt::skip]
        let moves = [
            Move::Normal { from: Square::SQ_1A, to: Square::SQ_2B, promote: false },
            Move::Normal { from: Square::SQ_5A, to: Square::SQ_5B, promote: false },
            Move::Normal { from: Square::SQ_2B, to: Square::SQ_1C, promote: false },
            Move::Normal { from: Square::SQ_5B, to: Square::SQ_5A, promote: false },
            Move::Normal { from: Square::SQ_1C, to: Square::SQ_2D, promote: false },
            Move::Normal { from: Square::SQ_5A, to: Square::SQ_5B, promote: false },
            Move::Normal { from: Square::SQ_2D, to: Square::SQ_1E, promote: false },
            Move::Normal { from: Square::SQ_5B, to: Square::SQ_5A, promote: false },
            Move::Normal { from: Square::SQ_1E, to: Square::SQ_2F, promote: false },
            Move::Normal { from: Square::SQ_5A, to: Square::SQ_5B, promote: false },
            Move::Normal { from: Square::SQ_2F, to: Square::SQ_1G, promote: false },
            Move::Normal { from: Square::SQ_5B, to: Square::SQ_5A, promote: false },
            Move::Normal { from: Square::SQ_1G, to: Square::SQ_2H, promote: false },
            Move::Normal { from: Square::SQ_5A, to: Square::SQ_5B, promote: false },
            Move::Normal { from: Square::SQ_2H, to: Square::SQ_3I, promote: false },
            Move::Normal { from: Square::SQ_5B, to: Square::SQ_5A, promote: false },
            Move::Normal { from: Square::SQ_3I, to: Square::SQ_4H, promote: false },
            Move::Normal { from: Square::SQ_5A, to: Square::SQ_5B, promote: false },
            Move::Normal { from: Square::SQ_4H, to: Square::SQ_5G, promote: false },
            Move::Normal { from: Square::SQ_5B, to: Square::SQ_5A, promote: false },
            Move::Normal { from: Square::SQ_5G, to: Square::SQ_6F, promote: false },
            Move::Normal { from: Square::SQ_5A, to: Square::SQ_5B, promote: false },
            Move::Normal { from: Square::SQ_6F, to: Square::SQ_7E, promote: false },
            Move::Normal { from: Square::SQ_5B, to: Square::SQ_5A, promote: false },
            Move::Normal { from: Square::SQ_7E, to: Square::SQ_8D, promote: false },
            Move::Normal { from: Square::SQ_5A, to: Square::SQ_5B, promote: false },
            Move::Normal { from: Square::SQ_8D, to: Square::SQ_9E, promote: false },
            Move::Normal { from: Square::SQ_5B, to: Square::SQ_5A, promote: false },
            Move::Normal { from: Square::SQ_9E, to: Square::SQ_8F, promote: false },
            Move::Normal { from: Square::SQ_5A, to: Square::SQ_5B, promote: false },
            Move::Normal { from: Square::SQ_8F, to: Square::SQ_9G, promote: false },
            Move::Normal { from: Square::SQ_5B, to: Square::SQ_5A, promote: false },
            Move::Normal { from: Square::SQ_9G, to: Square::SQ_8H, promote: false },
            Move::Normal { from: Square::SQ_5A, to: Square::SQ_5B, promote: false },
            Move::Normal { from: Square::SQ_8H, to: Square::SQ_9I, promote: false },
        ];
        for (i, &m) in moves.iter().enumerate() {
            assert!(pos.legal_moves().contains(&m), "move {:?} is not legal", m);
            pos.do_move(m);
            assert_eq!(
                i as u8 / 2 + 1,
                pos.hand(Color::Black)
                    .count(PieceKind::Pawn)
                    .unwrap_or_default()
            );
        }
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
                Move::Normal {
                    from: Square::SQ_7G,
                    to: Square::SQ_7F,
                    promote: false,
                },
                Move::Normal {
                    from: Square::SQ_3C,
                    to: Square::SQ_3D,
                    promote: false,
                },
                Move::Normal {
                    from: Square::SQ_2G,
                    to: Square::SQ_2F,
                    promote: false,
                },
            ];
            moves.iter().for_each(|&m| {
                pos.do_move(m);
            });
            pos.key()
        };
        let key1 = {
            let mut pos = Position::default();
            // +2726FU,-3334FU,+7776FU
            let moves = [
                Move::Normal {
                    from: Square::SQ_2G,
                    to: Square::SQ_2F,
                    promote: false,
                },
                Move::Normal {
                    from: Square::SQ_7G,
                    to: Square::SQ_7F,
                    promote: false,
                },
                Move::Normal {
                    from: Square::SQ_3C,
                    to: Square::SQ_3D,
                    promote: false,
                },
            ];
            moves.iter().for_each(|&m| {
                pos.do_move(m);
            });
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
                Move::Normal {
                    from: Square::SQ_7G,
                    to: Square::SQ_7F,
                    promote: false,
                },
                Move::Normal {
                    from: Square::SQ_3C,
                    to: Square::SQ_3D,
                    promote: false,
                },
                Move::Normal {
                    from: Square::SQ_8H,
                    to: Square::SQ_2B,
                    promote: false,
                },
                Move::Normal {
                    from: Square::SQ_3A,
                    to: Square::SQ_2B,
                    promote: false,
                },
                Move::Drop {
                    to: Square::SQ_8H,
                    piece: Piece::B_B,
                },
                Move::Normal {
                    from: Square::SQ_2B,
                    to: Square::SQ_3A,
                    promote: false,
                },
            ];
            moves.iter().for_each(|&m| {
                pos.do_move(m);
            });
            pos.keys()
        };
        let keys1 = {
            let mut pos = Position::default();
            // +7776FU,-3334FU,+8822KA,-3142GI,+2288KA,-4231GI
            // => P+00KA
            let moves = [
                Move::Normal {
                    from: Square::SQ_7G,
                    to: Square::SQ_7F,
                    promote: false,
                },
                Move::Normal {
                    from: Square::SQ_3C,
                    to: Square::SQ_3D,
                    promote: false,
                },
                Move::Normal {
                    from: Square::SQ_8H,
                    to: Square::SQ_2B,
                    promote: false,
                },
                Move::Normal {
                    from: Square::SQ_3A,
                    to: Square::SQ_4B,
                    promote: false,
                },
                Move::Normal {
                    from: Square::SQ_2B,
                    to: Square::SQ_8H,
                    promote: false,
                },
                Move::Normal {
                    from: Square::SQ_4B,
                    to: Square::SQ_3A,
                    promote: false,
                },
            ];
            moves.iter().for_each(|&m| {
                pos.do_move(m);
            });
            pos.keys()
        };
        assert_ne!(keys0, keys1);
        assert_eq!(keys0.0, keys1.0)
    }
}
