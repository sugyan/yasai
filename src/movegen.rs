use crate::tables::{ATTACK_TABLE, BETWEEN_TABLE, FILES, PROMOTABLE, RANKS, RELATIVE_RANKS};
use crate::Position;
use arrayvec::{ArrayVec, IntoIter};
use shogi_core::{Bitboard, Color, Move, Piece, PieceKind, Square};

pub struct MoveList(ArrayVec<Move, { MoveList::MAX_LEGAL_MOVES }>);

impl MoveList {
    const MAX_LEGAL_MOVES: usize = 593;

    pub fn generate_legals(&mut self, pos: &Position) {
        if pos.in_check() {
            self.generate_evasions(pos);
        } else {
            self.generate_all(pos);
        }

        let (mut i, mut size) = (0, self.0.len());
        while i != size {
            if Self::is_legal(pos, self.0[i]) {
                i += 1;
            } else {
                size -= 1;
                self.0.swap_remove(i);
            }
        }
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    fn generate_all(&mut self, pos: &Position) {
        let target = !pos.player_bitboard(pos.side_to_move());
        self.generate_for_fu(pos, &target);
        self.generate_for_ky(pos, &target);
        self.generate_for_ke(pos, &target);
        self.generate_for_gi(pos, &target);
        self.generate_for_ka(pos, &target);
        self.generate_for_hi(pos, &target);
        self.generate_for_ki(pos, &target);
        self.generate_for_ou(pos, &target);
        self.generate_for_um(pos, &target);
        self.generate_for_ry(pos, &target);
        self.generate_drop(pos, &(!pos.occupied() & !Bitboard::empty()));
    }
    fn generate_evasions(&mut self, pos: &Position) {
        let c = pos.side_to_move();
        if let Some(king) = pos.king_position(c) {
            let mut checkers_attacks = Bitboard::empty();
            let mut checkers_count = 0;
            for ch in pos.checkers() {
                if let Some(p) = pos.piece_at(ch) {
                    let pk = p.piece_kind();
                    // 龍が斜め位置から王手している場合のみ、他の駒の裏に逃がれることができる可能性がある
                    if pk == PieceKind::ProRook
                        && ch.file() != king.file()
                        && ch.rank() != king.rank()
                    {
                        checkers_attacks |= ATTACK_TABLE.hi.attack(ch, &pos.occupied());
                    } else {
                        checkers_attacks |= ATTACK_TABLE.pseudo_attack(pk, ch, c.flip());
                    }
                }
                checkers_count += 1;
            }
            for to in ATTACK_TABLE.ou.attack(king, c) & !pos.player_bitboard(c) & !checkers_attacks
            {
                self.push(Move::Normal {
                    from: king,
                    to,
                    promote: false,
                });
            }
            // 両王手の場合は玉が逃げるしかない
            if checkers_count > 1 {
                return;
            }
            if let Some(ch) = pos.checkers().pop() {
                let target_drop = BETWEEN_TABLE[ch.array_index()][king.array_index()];
                let target_move = target_drop | pos.checkers();
                self.generate_for_fu(pos, &target_move);
                self.generate_for_ky(pos, &target_move);
                self.generate_for_ke(pos, &target_move);
                self.generate_for_gi(pos, &target_move);
                self.generate_for_ka(pos, &target_move);
                self.generate_for_hi(pos, &target_move);
                self.generate_for_ki(pos, &target_move);
                self.generate_for_um(pos, &target_move);
                self.generate_for_ry(pos, &target_move);
                self.generate_drop(pos, &target_drop);
            }
        }
    }
    fn push(&mut self, m: Move) {
        unsafe { self.0.push_unchecked(m) };
    }
    fn generate_for_fu(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        let (to_bb, delta) = match c {
            Color::Black => (unsafe { pos.piece_bitboard(Piece::B_P).shift_up(1) }, 1),
            Color::White => (unsafe { pos.piece_bitboard(Piece::W_P).shift_down(1) }, !0),
        };
        for to in to_bb & *target {
            let from = unsafe { Square::from_u8_unchecked(to.index().wrapping_add(delta)) };
            if PROMOTABLE[to.array_index()][c.array_index()] {
                self.push(Move::Normal {
                    from,
                    to,
                    promote: true,
                });
                if RELATIVE_RANKS[to.array_index()][c.array_index()] > 1 {
                    self.push(Move::Normal {
                        from,
                        to,
                        promote: false,
                    });
                }
            } else {
                self.push(Move::Normal {
                    from,
                    to,
                    promote: false,
                });
            }
        }
    }
    fn generate_for_ky(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        for from in pos.player_bitboard(c) & pos.piece_kind_bitboard(PieceKind::Lance) {
            for to in ATTACK_TABLE.ky.attack(from, c, &pos.occupied()) & *target {
                if PROMOTABLE[to.array_index()][c.array_index()] {
                    self.push(Move::Normal {
                        from,
                        to,
                        promote: true,
                    });
                    if RELATIVE_RANKS[to.array_index()][c.array_index()] > 1 {
                        self.push(Move::Normal {
                            from,
                            to,
                            promote: false,
                        });
                    }
                } else {
                    self.push(Move::Normal {
                        from,
                        to,
                        promote: false,
                    });
                }
            }
        }
    }
    fn generate_for_ke(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        for from in pos.player_bitboard(c) & pos.piece_kind_bitboard(PieceKind::Knight) {
            for to in ATTACK_TABLE.ke.attack(from, c) & *target {
                if PROMOTABLE[to.array_index()][c.array_index()] {
                    self.push(Move::Normal {
                        from,
                        to,
                        promote: true,
                    });
                    if RELATIVE_RANKS[to.array_index()][c.array_index()] > 2 {
                        self.push(Move::Normal {
                            from,
                            to,
                            promote: false,
                        });
                    }
                } else {
                    self.push(Move::Normal {
                        from,
                        to,
                        promote: false,
                    });
                }
            }
        }
    }
    fn generate_for_gi(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        for from in pos.player_bitboard(c) & pos.piece_kind_bitboard(PieceKind::Silver) {
            let from_is_opponent_field = PROMOTABLE[from.array_index()][c.array_index()];
            for to in ATTACK_TABLE.gi.attack(from, c) & *target {
                self.push(Move::Normal {
                    from,
                    to,
                    promote: false,
                });
                if from_is_opponent_field || PROMOTABLE[to.array_index()][c.array_index()] {
                    self.push(Move::Normal {
                        from,
                        to,
                        promote: true,
                    });
                }
            }
        }
    }
    fn generate_for_ka(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        for from in pos.player_bitboard(c) & pos.piece_kind_bitboard(PieceKind::Bishop) {
            let from_is_opponent_field = PROMOTABLE[from.array_index()][c.array_index()];
            for to in ATTACK_TABLE.ka.attack(from, &pos.occupied()) & *target {
                self.push(Move::Normal {
                    from,
                    to,
                    promote: false,
                });
                if from_is_opponent_field || PROMOTABLE[to.array_index()][c.array_index()] {
                    self.push(Move::Normal {
                        from,
                        to,
                        promote: true,
                    });
                }
            }
        }
    }
    fn generate_for_hi(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        for from in pos.player_bitboard(c) & pos.piece_kind_bitboard(PieceKind::Rook) {
            let from_is_opponent_field = PROMOTABLE[from.array_index()][c.array_index()];
            for to in ATTACK_TABLE.hi.attack(from, &pos.occupied()) & *target {
                self.push(Move::Normal {
                    from,
                    to,
                    promote: false,
                });
                if from_is_opponent_field || PROMOTABLE[to.array_index()][c.array_index()] {
                    self.push(Move::Normal {
                        from,
                        to,
                        promote: true,
                    });
                }
            }
        }
    }
    fn generate_for_ki(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        for from in (pos.piece_kind_bitboard(PieceKind::Gold)
            | pos.piece_kind_bitboard(PieceKind::ProPawn)
            | pos.piece_kind_bitboard(PieceKind::ProLance)
            | pos.piece_kind_bitboard(PieceKind::ProKnight)
            | pos.piece_kind_bitboard(PieceKind::ProSilver))
            & pos.player_bitboard(c)
        {
            for to in ATTACK_TABLE.ki.attack(from, c) & *target {
                self.push(Move::Normal {
                    from,
                    to,
                    promote: false,
                });
            }
        }
    }
    fn generate_for_ou(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        for from in pos.player_bitboard(c) & pos.piece_kind_bitboard(PieceKind::King) {
            for to in ATTACK_TABLE.ou.attack(from, c) & *target {
                self.push(Move::Normal {
                    from,
                    to,
                    promote: false,
                });
            }
        }
    }
    fn generate_for_um(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        for from in pos.player_bitboard(c) & pos.piece_kind_bitboard(PieceKind::ProBishop) {
            for to in (ATTACK_TABLE.ka.attack(from, &pos.occupied())
                | ATTACK_TABLE.ou.attack(from, c))
                & *target
            {
                self.push(Move::Normal {
                    from,
                    to,
                    promote: false,
                });
            }
        }
    }
    fn generate_for_ry(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        for from in pos.player_bitboard(c) & pos.piece_kind_bitboard(PieceKind::ProRook) {
            for to in (ATTACK_TABLE.hi.attack(from, &pos.occupied())
                | ATTACK_TABLE.ou.attack(from, c))
                & *target
            {
                self.push(Move::Normal {
                    from,
                    to,
                    promote: false,
                });
            }
        }
    }
    fn generate_drop(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        let hand = pos.hand(pos.side_to_move());
        for pk in PieceKind::all() {
            if hand.count(pk).unwrap_or_default() == 0 {
                continue;
            }
            let mut exclude = Bitboard::empty();
            if pk == PieceKind::Pawn {
                // 二歩
                for sq in pos.player_bitboard(c) & pos.piece_kind_bitboard(pk) {
                    exclude |= FILES[sq.file() as usize];
                }
                // 打ち歩詰めチェック
                if let Some(sq) = pos.king_position(c.flip()) {
                    if let Some(to) = ATTACK_TABLE.fu.attack(sq, c.flip()).pop() {
                        if target.contains(to) && Self::is_uchifuzume(pos, to) {
                            exclude |= to;
                        }
                    }
                }
                exclude |= match c {
                    Color::Black => RANKS[1],
                    Color::White => RANKS[9],
                };
            }
            for to in *target & !exclude {
                if match pk {
                    PieceKind::Pawn | PieceKind::Lance => {
                        RELATIVE_RANKS[to.array_index()][c.array_index()] > 1
                    }
                    PieceKind::Knight => RELATIVE_RANKS[to.array_index()][c.array_index()] > 2,
                    _ => true,
                } {
                    self.push(Move::Drop {
                        to,
                        piece: Piece::new(pk, c),
                    });
                }
            }
        }
    }
    fn is_legal(pos: &Position, m: Move) -> bool {
        if let Some(from) = m.from() {
            let c = pos.side_to_move();
            let king = match c {
                Color::Black => Piece::B_K,
                Color::White => Piece::W_K,
            };
            // 玉が相手の攻撃範囲内に動いてしまう指し手は除外
            if pos.piece_at(from) == Some(king)
                && !pos
                    .attackers_to(c.flip(), m.to(), &pos.occupied())
                    .is_empty()
            {
                return false;
            }
            // 飛び駒から守っている駒が直線上から外れてしまう指し手は除外
            if pos.pinned(c).contains(from) {
                if let Some(sq) = pos.king_position(c) {
                    if !(BETWEEN_TABLE[sq.array_index()][from.array_index()].contains(m.to())
                        || BETWEEN_TABLE[sq.array_index()][m.to().array_index()].contains(from))
                    {
                        return false;
                    }
                }
            }
        }
        true
    }
    fn is_uchifuzume(pos: &Position, sq: Square) -> bool {
        let c = pos.side_to_move();
        // 玉自身が歩を取れる
        if pos.attackers_to(c, sq, &pos.occupied()).is_empty() {
            return false;
        }
        // 他の駒が歩を取れる
        let capture_candidates = Self::attackers_to_except_klp(pos, c.flip(), sq);
        if !(capture_candidates & !pos.pinned(c.flip())).is_empty() {
            return false;
        }
        // 玉が逃げられる
        if let Some(king) = pos.king_position(c.flip()) {
            let escape = ATTACK_TABLE.ou.attack(king, c.flip())
                & !pos.player_bitboard(c.flip())
                & !Bitboard::single(sq);
            let occupied = pos.occupied() | Bitboard::single(sq);
            for to in escape ^ Bitboard::single(sq) {
                if pos.attackers_to(c, to, &occupied).is_empty() {
                    return false;
                }
            }
        }
        true
    }
    #[rustfmt::skip]
    fn attackers_to_except_klp(pos: &Position, c: Color, to: Square) -> Bitboard {
        let opp = c.flip();
        let occ = &pos.occupied();
        (     (ATTACK_TABLE.ke.attack(to, opp) & pos.piece_kind_bitboard(PieceKind::Knight))
            | (ATTACK_TABLE.gi.attack(to, opp) & (pos.piece_kind_bitboard(PieceKind::Silver) | pos.piece_kind_bitboard(PieceKind::ProRook)))
            | (ATTACK_TABLE.ka.attack(to, occ) & (pos.piece_kind_bitboard(PieceKind::Bishop) | pos.piece_kind_bitboard(PieceKind::ProBishop)))
            | (ATTACK_TABLE.hi.attack(to, occ) & (pos.piece_kind_bitboard(PieceKind::Rook) | pos.piece_kind_bitboard(PieceKind::ProRook)))
            | (ATTACK_TABLE.ki.attack(to, opp) & (pos.piece_kind_bitboard(PieceKind::Gold) | pos.piece_kind_bitboard(PieceKind::ProPawn) | pos.piece_kind_bitboard(PieceKind::ProLance) | pos.piece_kind_bitboard(PieceKind::ProKnight) | pos.piece_kind_bitboard(PieceKind::ProSilver) | pos.piece_kind_bitboard(PieceKind::ProBishop)))
        ) & pos.player_bitboard(c)
    }
}

impl Default for MoveList {
    fn default() -> Self {
        Self(ArrayVec::new())
    }
}

impl IntoIterator for MoveList {
    type Item = Move;
    type IntoIter = IntoIter<Move, { MoveList::MAX_LEGAL_MOVES }>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shogi_core::PartialPosition;
    use shogi_usi_parser::FromUsi;

    #[test]
    fn from_default() {
        let pos = Position::default();
        assert_eq!(30, pos.legal_moves().len());
    }

    #[test]
    fn drop_moves() {
        // P1-KY-KE-GI-KI-OU-KI * -KE-KY
        // P2 * -HI *  *  *  *  * -GI *
        // P3-FU-FU-FU-FU-FU-FU * -FU-FU
        // P4 *  *  *  *  *  * -FU *  *
        // P5 *  *  *  *  *  *  *  *  *
        // P6 *  * +FU *  *  *  *  *  *
        // P7+FU+FU * +FU+FU+FU+FU+FU+FU
        // P8 *  *  *  *  *  *  * +HI *
        // P9+KY+KE+GI+GI+KI+OU+KI+GI+KE+KY
        // P+00KA
        // P-00KA
        // +
        let pos = Position::new(
            PartialPosition::from_usi(
                "sfen lnsgkg1nl/1r5s1/pppppp1pp/6p2/9/2P6/PP1PPPPPP/7R1/LNSGKGSNL b Bb 1",
            )
            .expect("failed to parse"),
        );
        assert_eq!(
            43,
            pos.legal_moves()
                .into_iter()
                .filter(|m| m.is_drop())
                .count()
        );
    }

    #[test]
    fn maximum_moves() {
        // http://lfics81.techblog.jp/archives/2041940.html
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
        let pos = Position::new(
            PartialPosition::from_usi("sfen R8/2K1S1SSk/4B4/9/9/9/9/9/1L1L1L3 b RBGSNLP3g3n17p 1")
                .expect("failed to parse"),
        );
        assert_eq!(593, pos.legal_moves().len());
    }

    #[allow(clippy::bool_assert_comparison)]
    #[test]
    fn is_uchifuzume() {
        let test_cases = [
            // 打ち歩詰め
            // P1 *  *  *  *  *  *  *  *  *
            // P2 *  *  *  *  *  *  * -FU-FU
            // P3 *  *  *  *  *  *  *  * -OU
            // P4 *  *  *  *  *  *  * +FU *
            // P5 *  *  *  *  *  *  * +KI *
            // P6 *  *  *  *  *  *  *  *  *
            // P7 *  *  *  *  *  *  *  *  *
            // P8 *  *  *  *  *  *  *  *  *
            // P9 *  *  *  *  *  *  *  *  *
            // P+00FU
            // P-00AL
            // +
            (
                Position::new(
                    PartialPosition::from_usi("sfen 9/7pp/8k/7P1/7G1/9/9/9/9 b P2r2b3g4s4n4l14p 1")
                        .expect("failed to parse"),
                ),
                Square::SQ_1D,
                true,
            ),
            // 金で歩を取れる
            // P1 *  *  *  *  *  *  *  *  *
            // P2 *  *  *  *  *  *  * -FU-FU
            // P3 *  *  *  *  *  *  * -KI-OU
            // P4 *  *  *  *  *  *  *  *  *
            // P5 *  *  *  *  *  *  * +KI *
            // P6 *  *  *  *  *  *  *  *  *
            // P7 *  *  *  *  *  *  *  *  *
            // P8 *  *  *  *  *  *  *  *  *
            // P9 *  *  *  *  *  *  *  *  *
            // P+00FU
            // P-00AL
            // +
            (
                Position::new(
                    PartialPosition::from_usi("sfen 9/7pp/7gk/9/7G1/9/9/9/9 b P2r2b2g4s4n4l15p 1")
                        .expect("failed to parse"),
                ),
                Square::SQ_1D,
                false,
            ),
            // 飛がいるので金が動けない
            // P1 *  *  *  *  *  *  *  *  *
            // P2 *  *  *  *  *  *  * -FU-FU
            // P3 *  *  *  *  *  * +HI-KI-OU
            // P4 *  *  *  *  *  *  *  *  *
            // P5 *  *  *  *  *  *  * +KI *
            // P6 *  *  *  *  *  *  *  *  *
            // P7 *  *  *  *  *  *  *  *  *
            // P8 *  *  *  *  *  *  *  *  *
            // P9 *  *  *  *  *  *  *  *  *
            // P+00FU
            // P-00AL
            // +
            (
                Position::new(
                    PartialPosition::from_usi("sfen 9/7pp/6Rgk/9/7G1/9/9/9/9 b Pr2b2g4s4n4l15p 1")
                        .expect("failed to parse"),
                ),
                Square::SQ_1D,
                true,
            ),
            // 桂で歩を取れる
            // P1 *  *  *  *  *  *  *  *  *
            // P2 *  *  *  *  *  *  * -KE-FU
            // P3 *  *  *  *  *  *  *  * -OU
            // P4 *  *  *  *  *  *  * +FU *
            // P5 *  *  *  *  *  *  * +KI *
            // P6 *  *  *  *  *  *  *  *  *
            // P7 *  *  *  *  *  *  *  *  *
            // P8 *  *  *  *  *  *  *  *  *
            // P9 *  *  *  *  *  *  *  *  *
            // P+00FU
            // P-00AL
            // +
            (
                Position::new(
                    PartialPosition::from_usi("sfen 9/7np/8k/7P1/7G1/9/9/9/9 b P2r2b3g4s3n4l15p 1")
                        .expect("failed to parse"),
                ),
                Square::SQ_1D,
                false,
            ),
            // 角がいるので桂が動けない
            // P1 *  *  *  *  *  * +KA *  *
            // P2 *  *  *  *  *  *  * -KE-FU
            // P3 *  *  *  *  *  *  *  * -OU
            // P4 *  *  *  *  *  *  * +FU *
            // P5 *  *  *  *  *  *  * +KI *
            // P6 *  *  *  *  *  *  *  *  *
            // P7 *  *  *  *  *  *  *  *  *
            // P8 *  *  *  *  *  *  *  *  *
            // P9 *  *  *  *  *  *  *  *  *
            // P+00FU
            // P-00AL
            // +
            (
                Position::new(
                    PartialPosition::from_usi(
                        "sfen 6B2/7np/8k/7P1/7G1/9/9/9/9 b P2rb3g4s3n4l15p 1",
                    )
                    .expect("failed to parse"),
                ),
                Square::SQ_1D,
                true,
            ),
            // https://github.com/nozaq/shogi-rs/pull/41
            // 打った歩によって△1一玉の逃げ場ができる
            // P1 *  *  *  *  *  *  * -OU *
            // P2 *  *  *  *  * +KI *  * -KY
            // P3 *  *  *  *  *  * +KA *  *
            // P4 *  *  *  *  *  *  *  *  *
            // P5 *  *  *  *  *  *  *  *  *
            // P6 *  *  *  *  *  *  *  *  *
            // P7 *  *  *  *  *  *  *  *  *
            // P8 *  *  *  *  *  *  *  *  *
            // P9 *  *  *  *  *  *  *  *  *
            // P+00FU
            // P-00AL
            // +
            (
                Position::new(
                    PartialPosition::from_usi("sfen 7k1/5G2l/6B2/9/9/9/9/9/9 b P2rb3g4s4n3l17p 1")
                        .expect("failed to parse"),
                ),
                Square::SQ_2B,
                false,
            ),
        ];
        for (pos, sq, expected) in test_cases {
            assert_eq!(expected, MoveList::is_uchifuzume(&pos, sq));
        }
    }
}
