use crate::movegen::MoveList;
use crate::state::{AttackInfo, State};
use crate::tables::{ATTACK_TABLE, BETWEEN_TABLE};
use crate::zobrist::{Key, ZOBRIST_TABLE};
use shogi_core::{Bitboard, Color, Hand, Move, PartialPosition, Piece, PieceKind, Square};

/// Represents a state of the game.
#[derive(Debug, Clone)]
pub struct Position {
    inner: PartialPosition,
    states: Vec<State>,
}

impl Position {
    pub fn new(partial: PartialPosition) -> Position {
        let mut keys = (Key::ZERO, Key::ZERO);
        for sq in Square::all() {
            if let Some(p) = partial.piece_at(sq) {
                keys.0 ^= ZOBRIST_TABLE.board(sq, p);
            }
        }
        for p in Piece::all() {
            if let Some(num) = partial.hand(p) {
                for i in 0..num {
                    keys.1 ^= ZOBRIST_TABLE.hand(p.color(), p.piece_kind(), i);
                }
            }
        }
        let checkers = AttackInfo::calculate_checkers(&partial);
        let state = State::new(keys, None, None, AttackInfo::new(checkers, &partial));
        Self {
            inner: partial,
            states: vec![state],
        }
    }
    pub fn hand(&self, c: Color) -> Hand {
        self.inner.hand_of_a_player(c)
    }
    pub fn side_to_move(&self) -> Color {
        self.inner.side_to_move()
    }
    pub fn ply(&self) -> u16 {
        self.inner.ply()
    }
    pub fn piece_at(&self, sq: Square) -> Option<Piece> {
        self.inner.piece_at(sq)
    }
    pub(crate) fn player_bitboard(&self, c: Color) -> Bitboard {
        self.inner.player_bitboard(c)
    }
    pub(crate) fn piece_kind_bitboard(&self, pk: PieceKind) -> Bitboard {
        self.inner.piece_kind_bitboard(pk)
    }
    pub(crate) fn piece_bitboard(&self, p: Piece) -> Bitboard {
        self.inner.piece_bitboard(p)
    }
    pub(crate) fn occupied(&self) -> Bitboard {
        !self.inner.vacant_bitboard()
    }
    pub fn key(&self) -> u64 {
        self.state().key().value()
    }
    pub fn keys(&self) -> (u64, u64) {
        (self.state().keys().0.value(), self.state().keys().1.value())
    }
    pub fn in_check(&self) -> bool {
        !self.checkers().is_empty()
    }
    pub(crate) fn captured(&self) -> Option<Piece> {
        self.state().captured()
    }
    pub(crate) fn last_moved(&self) -> Option<Piece> {
        self.state().last_moved()
    }
    pub(crate) fn checkers(&self) -> Bitboard {
        self.state().attack_info().checkers()
    }
    pub(crate) fn pinned(&self, c: Color) -> Bitboard {
        self.state().attack_info().pinned(c)
    }
    pub(crate) fn king_position(&self, c: Color) -> Option<Square> {
        self.inner.king_position(c)
    }
    fn checkable(&self, pk: PieceKind, sq: Square) -> bool {
        self.state().attack_info().checkable(pk, sq)
    }
    pub fn legal_moves(&self) -> MoveList {
        let mut ml = MoveList::default();
        ml.generate_legals(self);
        ml
    }
    pub fn do_move(&mut self, m: Move) -> Option<()> {
        let is_check = self.is_check_move(m);
        let captured = self.inner.piece_at(m.to());
        let last_moved;
        let mut keys = self.state().keys();
        let checkers = match m {
            Move::Normal { from, to, promote } => {
                let piece = self.inner.piece_at(from)?;
                last_moved = Some(piece);
                if let Some(p) = captured {
                    let pk = p.piece_kind();
                    let pk_unpromoted = if let Some(pk) = pk.unpromote() {
                        pk
                    } else {
                        pk
                    };
                    keys.0 ^= ZOBRIST_TABLE.board(to, p);
                    keys.1 ^= ZOBRIST_TABLE.hand(
                        self.side_to_move(),
                        pk_unpromoted,
                        self.inner
                            .hand_of_a_player(self.side_to_move())
                            .count(pk_unpromoted)?,
                    );
                }
                let target_piece = if promote { piece.promote()? } else { piece };
                keys.0 ^= ZOBRIST_TABLE.board(from, piece);
                keys.0 ^= ZOBRIST_TABLE.board(to, target_piece);
                self.inner.make_move(m);
                if is_check {
                    AttackInfo::calculate_checkers(&self.inner)
                } else {
                    Bitboard::empty()
                }
            }
            Move::Drop { to, piece } => {
                last_moved = Some(piece);
                keys.1 ^= ZOBRIST_TABLE.hand(
                    self.side_to_move(),
                    piece.piece_kind(),
                    self.inner
                        .hand_of_a_player(self.side_to_move())
                        .count(piece.piece_kind())?,
                );
                keys.0 ^= ZOBRIST_TABLE.board(to, piece);
                self.inner.make_move(m);
                if is_check {
                    Bitboard::single(to)
                } else {
                    Bitboard::empty()
                }
            }
        };
        keys.0 ^= Key::COLOR;
        self.states.push(State::new(
            keys,
            captured,
            last_moved,
            AttackInfo::new(checkers, &self.inner),
        ));
        Some(())
    }
    pub fn undo_move(&mut self, m: Move) -> Option<()> {
        match m {
            Move::Normal {
                from,
                to,
                promote: _,
            } => {
                let piece = self.last_moved()?;
                self.inner.piece_set(to, None);
                if let Some(p_cap) = self.captured() {
                    let pk = p_cap.piece_kind();
                    self.inner.piece_set(to, Some(p_cap));
                    let pk_unpromoted = if let Some(pk) = pk.unpromote() {
                        pk
                    } else {
                        pk
                    };
                    *self.inner.hand_of_a_player_mut(self.side_to_move().flip()) = self
                        .inner
                        .hand_of_a_player(self.side_to_move().flip())
                        .removed(pk_unpromoted)?;
                }
                self.inner.piece_set(from, Some(piece));
            }
            Move::Drop { to, piece } => {
                self.inner.piece_set(to, None);
                *self.inner.hand_of_a_player_mut(self.side_to_move().flip()) = self
                    .inner
                    .hand_of_a_player(self.side_to_move().flip())
                    .added(piece.piece_kind())?;
            }
        }
        self.inner.side_to_move_set(self.side_to_move().flip());
        if !self.inner.ply_set(self.inner.ply() - 1) {
            return None;
        };
        self.states.pop();
        Some(())
    }
    fn state(&self) -> &State {
        self.states.last().expect("empty states")
    }
    #[rustfmt::skip]
    pub fn attackers_to(&self, c: Color, to: Square, occ: &Bitboard) -> Bitboard {
        let opp = c.flip();
        (     (ATTACK_TABLE.fu.attack(to, opp)      & self.piece_kind_bitboard(PieceKind::Pawn))
            | (ATTACK_TABLE.ky.attack(to, opp, occ) & self.piece_kind_bitboard(PieceKind::Lance))
            | (ATTACK_TABLE.ke.attack(to, opp)      & self.piece_kind_bitboard(PieceKind::Knight))
            | (ATTACK_TABLE.gi.attack(to, opp)      & (self.piece_kind_bitboard(PieceKind::Silver) | self.piece_kind_bitboard(PieceKind::ProRook) | self.piece_kind_bitboard(PieceKind::King)))
            | (ATTACK_TABLE.ka.attack(to, occ)      & (self.piece_kind_bitboard(PieceKind::Bishop) | self.piece_kind_bitboard(PieceKind::ProBishop)))
            | (ATTACK_TABLE.hi.attack(to, occ)      & (self.piece_kind_bitboard(PieceKind::Rook) | self.piece_kind_bitboard(PieceKind::ProRook)))
            | (ATTACK_TABLE.ki.attack(to, opp)      & (self.piece_kind_bitboard(PieceKind::Gold) | self.piece_kind_bitboard(PieceKind::ProPawn) | self.piece_kind_bitboard(PieceKind::ProLance) | self.piece_kind_bitboard(PieceKind::ProKnight) | self.piece_kind_bitboard(PieceKind::ProSilver) | self.piece_kind_bitboard(PieceKind::ProBishop) | self.piece_kind_bitboard(PieceKind::King)))
        ) & self.player_bitboard(c)
    }
    pub fn is_check_move(&self, m: Move) -> bool {
        match m {
            Move::Normal { from, to, promote } => {
                let piece = self.inner.piece_at(from).expect("piece does not exist");
                let p = if promote {
                    if let Some(p) = piece.promote() {
                        p
                    } else {
                        piece
                    }
                } else {
                    piece
                };
                if self.checkable(p.piece_kind(), to) {
                    return true;
                }
                // 開き王手
                let c = self.inner.side_to_move();
                if self.pinned(c.flip()).contains(from) {
                    if let Some(sq) = self.inner.king_position(c.flip()) {
                        return !(BETWEEN_TABLE[sq.array_index()][from.array_index()].contains(to)
                            || BETWEEN_TABLE[sq.array_index()][to.array_index()].contains(from));
                    }
                }
                false
            }
            Move::Drop { to, piece } => self.checkable(piece.piece_kind(), to),
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new(PartialPosition::startpos())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shogi_usi_parser::FromUsi;

    #[test]
    fn default() {
        let pos = Position::default();
        for sq in Square::all() {
            #[rustfmt::skip]
            let expected = match sq {
                Square::SQ_1G | Square::SQ_2G | Square::SQ_3G | Square::SQ_4G | Square::SQ_5G | Square::SQ_6G | Square::SQ_7G | Square::SQ_8G | Square::SQ_9G => Some(Piece::B_P),
                Square::SQ_1I | Square::SQ_9I => Some(Piece::B_L),
                Square::SQ_2I | Square::SQ_8I => Some(Piece::B_N),
                Square::SQ_3I | Square::SQ_7I => Some(Piece::B_S),
                Square::SQ_4I | Square::SQ_6I => Some(Piece::B_G),
                Square::SQ_5I => Some(Piece::B_K),
                Square::SQ_8H => Some(Piece::B_B),
                Square::SQ_2H => Some(Piece::B_R),
                Square::SQ_1C | Square::SQ_2C | Square::SQ_3C | Square::SQ_4C | Square::SQ_5C | Square::SQ_6C | Square::SQ_7C | Square::SQ_8C | Square::SQ_9C => Some(Piece::W_P),
                Square::SQ_1A | Square::SQ_9A => Some(Piece::W_L),
                Square::SQ_2A | Square::SQ_8A => Some(Piece::W_N),
                Square::SQ_3A | Square::SQ_7A => Some(Piece::W_S),
                Square::SQ_4A | Square::SQ_6A => Some(Piece::W_G),
                Square::SQ_5A => Some(Piece::W_K),
                Square::SQ_2B => Some(Piece::W_B),
                Square::SQ_8B => Some(Piece::W_R),
                _ => None,
            };
            assert_eq!(expected, pos.piece_at(sq), "square: {:?}", sq);
        }
        for c in Color::all() {
            for pk in PieceKind::all() {
                if let Some(num) = pos.hand(c).count(pk) {
                    assert_eq!(0, num);
                }
            }
        }
        assert_eq!(Color::Black, pos.side_to_move());
        assert_eq!(1, pos.ply());
        assert!(!pos.in_check());
    }

    #[allow(clippy::bool_assert_comparison)]
    #[test]
    fn do_undo_move() {
        let mut pos = Position::default();
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
                promote: true,
            },
            Move::Normal {
                from: Square::SQ_3A,
                to: Square::SQ_2B,
                promote: false,
            },
            Move::Drop {
                to: Square::SQ_3C,
                piece: Piece::B_B,
            },
        ];
        // do moves
        for &m in moves.iter() {
            pos.do_move(m).expect("illegal move");
        }
        // check moved pieces, position states
        for (sq, expected) in [
            (Square::SQ_2B, Some(Piece::W_S)),
            (Square::SQ_3A, None),
            (Square::SQ_3C, Some(Piece::B_B)),
            (Square::SQ_7F, Some(Piece::B_P)),
            (Square::SQ_7G, None),
        ] {
            assert_eq!(expected, pos.piece_at(sq), "square: {:?}", sq);
        }
        assert_eq!(
            0,
            PieceKind::all()
                .iter()
                .filter_map(|&pk| pos.hand(Color::Black).count(pk))
                .sum::<u8>()
        );
        assert_ne!(
            0,
            PieceKind::all()
                .iter()
                .filter_map(|&pk| pos.hand(Color::White).count(pk))
                .sum::<u8>()
        );
        assert_eq!(Color::White, pos.side_to_move());
        assert_eq!(6, pos.ply());
        assert_eq!(true, pos.in_check());
        // revert to default position
        for &m in moves.iter().rev() {
            pos.undo_move(m).expect("illegal move");
        }
        let default = Position::default();
        assert!(Square::all().all(|sq| pos.piece_at(sq) == default.piece_at(sq)));
        assert_eq!(Color::Black, pos.side_to_move());
        assert_eq!(1, pos.ply());
        assert_eq!(false, pos.in_check());
    }

    #[test]
    fn perft() {
        fn perft(pos: &mut Position, depth: usize) -> u64 {
            if depth == 0 {
                return 1;
            }
            let mut count = 0;
            for m in pos.legal_moves() {
                pos.do_move(m).expect("illegal move");
                count += perft(pos, depth - 1);
                pos.undo_move(m).expect("illegal move");
            }
            count
        }

        // from default position
        {
            let mut pos = Position::default();
            assert_eq!(30, perft(&mut pos, 1));
            assert_eq!(900, perft(&mut pos, 2));
            assert_eq!(25470, perft(&mut pos, 3));
            assert_eq!(719731, perft(&mut pos, 4));
        }
        // from maximum moves
        // https://qiita.com/ak11/items/8bd5f2bb0f5b014143c8#%E5%88%9D%E6%9C%9F%E5%B1%80%E9%9D%A2%E3%81%98%E3%82%83%E3%81%AA%E3%81%84%E5%B1%80%E9%9D%A2
        // P1+HI *  *  *  *  *  *  *  *
        // P2 *  * +OU * +GI * +GI+GI-OU
        // P3 *  *  *  * +KA *  *  *  *
        // P4 *  *  *  *  *  *  *  *  *
        // P5 *  *  *  *  *  *  *  *  *
        // P6 *  *  *  *  *  *  *  *  *
        // P7 *  *  *  *  *  *  *  *  *
        // P8 *  *  *  *  *  *  *  *  *
        // P9 * +KY * +KY * +KY *  *  *
        // P+00FU00KY00KE00GI00KI00KA00HI
        // P-00AL
        // +
        {
            let mut pos = Position::new(
                PartialPosition::from_usi(
                    "sfen R8/2K1S1SSk/4B4/9/9/9/9/9/1L1L1L3 b RBGSNLP3g3n17p 1",
                )
                .expect("failed to parse"),
            );
            assert_eq!(593, perft(&mut pos, 1));
            assert_eq!(105677, perft(&mut pos, 2));
        }
    }

    #[allow(clippy::bool_assert_comparison)]
    #[test]
    fn is_check_move() {
        // P1 *  *  *  *  *  * -FU * -OU
        // P2 *  *  *  *  *  *  *  *  *
        // P3 *  *  *  *  *  * +FU * +KI
        // P4 *  *  *  *  *  *  *  *  *
        // P5 *  *  *  *  *  *  *  * +KY
        // P6 *  *  *  *  *  *  *  *  *
        // P7 *  *  *  *  *  *  *  *  *
        // P8 *  *  *  *  *  *  *  *  *
        // P9 *  *  *  *  *  *  *  *  *
        // P-00AL
        // +
        let pos = Position::new(
            PartialPosition::from_usi("sfen 6p1k/9/6P1G/9/8L/9/9/9/9 b RBLrb3g4s4n2l16p 1")
                .expect("failed to parse"),
        );
        let test_cases = [
            (
                Move::Drop {
                    to: Square::SQ_1B,
                    piece: Piece::B_L,
                },
                true,
            ),
            (
                Move::Drop {
                    to: Square::SQ_1D,
                    piece: Piece::B_L,
                },
                false,
            ),
            (
                Move::Drop {
                    to: Square::SQ_2B,
                    piece: Piece::B_B,
                },
                true,
            ),
            (
                Move::Drop {
                    to: Square::SQ_5E,
                    piece: Piece::B_B,
                },
                false,
            ),
            (
                Move::Drop {
                    to: Square::SQ_2A,
                    piece: Piece::B_R,
                },
                true,
            ),
            (
                Move::Drop {
                    to: Square::SQ_5A,
                    piece: Piece::B_R,
                },
                false,
            ),
            (
                Move::Normal {
                    from: Square::SQ_1C,
                    to: Square::SQ_1B,
                    promote: false,
                },
                true,
            ),
            (
                Move::Normal {
                    from: Square::SQ_1C,
                    to: Square::SQ_2B,
                    promote: false,
                },
                true,
            ),
            (
                Move::Normal {
                    from: Square::SQ_1C,
                    to: Square::SQ_2C,
                    promote: false,
                },
                true,
            ),
            (
                Move::Normal {
                    from: Square::SQ_1C,
                    to: Square::SQ_1D,
                    promote: false,
                },
                false,
            ),
        ];
        for (m, expected) in test_cases {
            assert_eq!(expected, pos.is_check_move(m));
        }
    }
}
