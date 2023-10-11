use crate::bitboard::{Bitboard, Occupied};
use crate::tables::{ATTACK_TABLE, BETWEEN_TABLE, FILES, PROMOTABLE, RELATIVE_RANKS};
use crate::Position;
use arrayvec::ArrayVec;
use shogi_core::{Color, Hand, Move, Piece, PieceKind, Square};

const MAX_LEGAL_MOVES: usize = 593;

impl Position {
    pub fn legal_moves(&self) -> ArrayVec<Move, MAX_LEGAL_MOVES> {
        let mut av = ArrayVec::new();
        if self.in_check() {
            self.generate_evasions(&mut av);
        } else {
            self.generate_all(&mut av);
        }

        let mut i = 0;
        while i != av.len() {
            if self.is_legal(av[i]) {
                i += 1;
            } else {
                av.swap_remove(i);
            }
        }
        av
    }
    /// Generate moves.
    fn generate_all(&self, av: &mut ArrayVec<Move, MAX_LEGAL_MOVES>) {
        let target = !self.player_bitboard(self.side_to_move());
        self.generate_for_fu(av, &target);
        self.generate_for_ky(av, &target);
        self.generate_for_ke(av, &target);
        self.generate_for_gi(av, &target);
        self.generate_for_ka(av, &target);
        self.generate_for_hi(av, &target);
        self.generate_for_ki(av, &target);
        self.generate_for_ou(av, &target);
        self.generate_for_um(av, &target);
        self.generate_for_ry(av, &target);
        self.generate_drop(av, &(!self.occupied_bitboard() & !Bitboard::empty()));
    }
    /// Generate moves to evade check, optimized using AttackInfo.
    fn generate_evasions(&self, av: &mut ArrayVec<Move, MAX_LEGAL_MOVES>) {
        let c = self.side_to_move();
        let king = self.king_position(c).unwrap();
        let mut checkers_attacks = Bitboard::empty();
        let mut checkers_count = 0;
        for ch in self.checkers() {
            let pk = self.piece_at(ch).unwrap().piece_kind();
            // 龍が斜め位置から王手している場合のみ、他の駒の裏に逃がれることができる可能性がある
            if pk == PieceKind::ProRook && ch.file() != king.file() && ch.rank() != king.rank() {
                checkers_attacks |= ATTACK_TABLE.hi.attack(ch, &self.occupied_bitboard());
            } else {
                checkers_attacks |= ATTACK_TABLE.pseudo_attack(pk, ch, c.flip());
            }
            checkers_count += 1;
        }
        for to in ATTACK_TABLE.ou.attack(king, c) & !self.player_bitboard(c) & !checkers_attacks {
            av.push(Move::Normal {
                from: king,
                to,
                promote: false,
            });
        }
        // 両王手の場合は玉が逃げるしかない
        if checkers_count > 1 {
            return;
        }
        let ch = self.checkers().into_iter().next().unwrap();
        let target_drop = BETWEEN_TABLE[ch.array_index()][king.array_index()];
        let target_move = target_drop | self.checkers();
        self.generate_for_fu(av, &target_move);
        self.generate_for_ky(av, &target_move);
        self.generate_for_ke(av, &target_move);
        self.generate_for_gi(av, &target_move);
        self.generate_for_ka(av, &target_move);
        self.generate_for_hi(av, &target_move);
        self.generate_for_ki(av, &target_move);
        self.generate_for_um(av, &target_move);
        self.generate_for_ry(av, &target_move);
        if !target_drop.is_empty() {
            // No need to exclude occupied bitboard: Existence of cells between attacker and king is given.
            self.generate_drop(av, &target_drop);
        }
    }
    fn generate_for_fu(&self, av: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, target: &Bitboard) {
        let c = self.side_to_move();
        let (to_bb, delta) = [
            (self.piece_bitboard(Piece::B_P).shr(), 1),
            (self.piece_bitboard(Piece::W_P).shl(), !0),
        ][c.array_index()];
        for to in to_bb & target {
            let from = unsafe { Square::from_u8_unchecked(to.index().wrapping_add(delta)) };
            if PROMOTABLE[to.array_index()][c.array_index()] {
                av.push(Move::Normal {
                    from,
                    to,
                    promote: true,
                });
                if RELATIVE_RANKS[to.array_index()][c.array_index()] > 1 {
                    av.push(Move::Normal {
                        from,
                        to,
                        promote: false,
                    });
                }
            } else {
                av.push(Move::Normal {
                    from,
                    to,
                    promote: false,
                });
            }
        }
    }
    fn generate_for_ky(&self, av: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, target: &Bitboard) {
        let c = self.side_to_move();
        for from in self.player_bitboard(c) & self.piece_kind_bitboard(PieceKind::Lance) {
            for to in ATTACK_TABLE.ky.attack(from, c, &self.occupied_bitboard()) & target {
                if PROMOTABLE[to.array_index()][c.array_index()] {
                    av.push(Move::Normal {
                        from,
                        to,
                        promote: true,
                    });
                    if RELATIVE_RANKS[to.array_index()][c.array_index()] > 1 {
                        av.push(Move::Normal {
                            from,
                            to,
                            promote: false,
                        });
                    }
                } else {
                    av.push(Move::Normal {
                        from,
                        to,
                        promote: false,
                    });
                }
            }
        }
    }
    fn generate_for_ke(&self, av: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, target: &Bitboard) {
        let c = self.side_to_move();
        for from in self.player_bitboard(c) & self.piece_kind_bitboard(PieceKind::Knight) {
            for to in ATTACK_TABLE.ke.attack(from, c) & target {
                if PROMOTABLE[to.array_index()][c.array_index()] {
                    av.push(Move::Normal {
                        from,
                        to,
                        promote: true,
                    });
                    if RELATIVE_RANKS[to.array_index()][c.array_index()] > 2 {
                        av.push(Move::Normal {
                            from,
                            to,
                            promote: false,
                        });
                    }
                } else {
                    av.push(Move::Normal {
                        from,
                        to,
                        promote: false,
                    });
                }
            }
        }
    }
    fn generate_for_gi(&self, av: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, target: &Bitboard) {
        let c = self.side_to_move();
        for from in self.player_bitboard(c) & self.piece_kind_bitboard(PieceKind::Silver) {
            let from_is_opponent_field = PROMOTABLE[from.array_index()][c.array_index()];
            for to in ATTACK_TABLE.gi.attack(from, c) & target {
                av.push(Move::Normal {
                    from,
                    to,
                    promote: false,
                });
                if from_is_opponent_field || PROMOTABLE[to.array_index()][c.array_index()] {
                    av.push(Move::Normal {
                        from,
                        to,
                        promote: true,
                    });
                }
            }
        }
    }
    fn generate_for_ka(&self, av: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, target: &Bitboard) {
        let c = self.side_to_move();
        for from in self.player_bitboard(c) & self.piece_kind_bitboard(PieceKind::Bishop) {
            let from_is_opponent_field = PROMOTABLE[from.array_index()][c.array_index()];
            for to in ATTACK_TABLE.ka.attack(from, &self.occupied_bitboard()) & target {
                av.push(Move::Normal {
                    from,
                    to,
                    promote: false,
                });
                if from_is_opponent_field || PROMOTABLE[to.array_index()][c.array_index()] {
                    av.push(Move::Normal {
                        from,
                        to,
                        promote: true,
                    });
                }
            }
        }
    }
    fn generate_for_hi(&self, av: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, target: &Bitboard) {
        let c = self.side_to_move();
        for from in self.player_bitboard(c) & self.piece_kind_bitboard(PieceKind::Rook) {
            let from_is_opponent_field = PROMOTABLE[from.array_index()][c.array_index()];
            for to in ATTACK_TABLE.hi.attack(from, &self.occupied_bitboard()) & target {
                av.push(Move::Normal {
                    from,
                    to,
                    promote: false,
                });
                if from_is_opponent_field || PROMOTABLE[to.array_index()][c.array_index()] {
                    av.push(Move::Normal {
                        from,
                        to,
                        promote: true,
                    });
                }
            }
        }
    }
    // Generate moves of pieces which moves like KI
    fn generate_for_ki(&self, av: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, target: &Bitboard) {
        let c = self.side_to_move();
        for from in (self.piece_kind_bitboard(PieceKind::Gold)
            | self.piece_kind_bitboard(PieceKind::ProPawn)
            | self.piece_kind_bitboard(PieceKind::ProLance)
            | self.piece_kind_bitboard(PieceKind::ProKnight)
            | self.piece_kind_bitboard(PieceKind::ProSilver))
            & self.player_bitboard(c)
        {
            for to in ATTACK_TABLE.ki.attack(from, c) & target {
                av.push(Move::Normal {
                    from,
                    to,
                    promote: false,
                });
            }
        }
    }
    fn generate_for_ou(&self, av: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, target: &Bitboard) {
        let c = self.side_to_move();
        for from in self.player_bitboard(c) & self.piece_kind_bitboard(PieceKind::King) {
            for to in ATTACK_TABLE.ou.attack(from, c) & target {
                av.push(Move::Normal {
                    from,
                    to,
                    promote: false,
                });
            }
        }
    }
    fn generate_for_um(&self, av: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, target: &Bitboard) {
        let c = self.side_to_move();
        for from in self.player_bitboard(c) & self.piece_kind_bitboard(PieceKind::ProBishop) {
            for to in (ATTACK_TABLE.ka.attack(from, &self.occupied_bitboard())
                | ATTACK_TABLE.ou.attack(from, c))
                & target
            {
                av.push(Move::Normal {
                    from,
                    to,
                    promote: false,
                });
            }
        }
    }
    fn generate_for_ry(&self, av: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, target: &Bitboard) {
        let c = self.side_to_move();
        for from in self.player_bitboard(c) & self.piece_kind_bitboard(PieceKind::ProRook) {
            for to in (ATTACK_TABLE.hi.attack(from, &self.occupied_bitboard())
                | ATTACK_TABLE.ou.attack(from, c))
                & target
            {
                av.push(Move::Normal {
                    from,
                    to,
                    promote: false,
                });
            }
        }
    }
    fn generate_drop(&self, av: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, target: &Bitboard) {
        let c = self.side_to_move();
        let hand = self.hand(self.side_to_move());
        for pk in Hand::all_hand_pieces().filter(|&pk| hand.count(pk).unwrap_or_default() > 0) {
            let mut target = *target;
            if pk == PieceKind::Pawn {
                target &= (self.player_bitboard(c) & self.piece_kind_bitboard(PieceKind::Pawn))
                    .vacant_files();
                // 打ち歩詰めチェック
                if let Some(sq) = self.king_position(c.flip()) {
                    if let Some(to) = ATTACK_TABLE.fu.attack(sq, c.flip()).into_iter().next() {
                        if target.contains(to) && self.is_pawn_drop_mate(to) {
                            target &= !Bitboard::single(to);
                        }
                    }
                }
            }
            let piece = Piece::new(pk, c);
            for to in target {
                if match pk {
                    PieceKind::Pawn | PieceKind::Lance => {
                        RELATIVE_RANKS[to.array_index()][c.array_index()] > 1
                    }
                    PieceKind::Knight => RELATIVE_RANKS[to.array_index()][c.array_index()] > 2,
                    _ => true,
                } {
                    av.push(Move::Drop { to, piece });
                }
            }
        }
    }
    // Checks if the move isn't illegal: king's suicidal moves and moving pinned piece away.
    fn is_legal(&self, m: Move) -> bool {
        if let Some(from) = m.from() {
            let c = self.side_to_move();
            let king = [Piece::B_K, Piece::W_K][c.array_index()];
            // 玉が相手の攻撃範囲内に動いてしまう指し手は除外
            if self.piece_at(from) == Some(king)
                && !self
                    .attackers_to(c.flip(), m.to(), &self.occupied_bitboard())
                    .is_empty()
            {
                return false;
            }
            // 飛び駒から守っている駒が直線上から外れてしまう指し手は除外
            if self.pinned(c).contains(from) {
                if let Some(sq) = self.king_position(c) {
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
    fn is_pawn_drop_mate(&self, sq: Square) -> bool {
        let c = self.side_to_move();
        // 玉自身が歩を取れる
        if self
            .attackers_to(c, sq, &self.occupied_bitboard())
            .is_empty()
        {
            return false;
        }
        // 他の駒が歩を取れる
        // 飛/龍がまっすぐ引いて取るのは常に可能
        let capture_candidates = self.attackers_to_except_klp(c.flip(), sq);
        if !(capture_candidates & (!self.pinned(c.flip()) | FILES[usize::from(sq.file())]))
            .is_empty()
        {
            return false;
        }
        // 玉が逃げられる
        if let Some(king) = self.king_position(c.flip()) {
            let single = Bitboard::single(sq);
            let escape =
                ATTACK_TABLE.ou.attack(king, c.flip()) & !self.player_bitboard(c.flip()) & !single;
            let occupied = self.occupied_bitboard() | single;
            for to in escape {
                if self.attackers_to(c, to, &occupied).is_empty() {
                    return false;
                }
            }
        }
        true
    }
    #[rustfmt::skip]
    fn attackers_to(&self, c: Color, to: Square, occ: &Bitboard) -> Bitboard {
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
    /// Attackers except for king, lance & pawn, which are not applicable to evade check by pawn
    #[rustfmt::skip]
    fn attackers_to_except_klp(&self, c: Color, to: Square) -> Bitboard {
        let opp = c.flip();
        let occ = &self.occupied_bitboard();
        (     (ATTACK_TABLE.ke.attack(to, opp) & self.piece_kind_bitboard(PieceKind::Knight))
            | (ATTACK_TABLE.gi.attack(to, opp) & (self.piece_kind_bitboard(PieceKind::Silver) | self.piece_kind_bitboard(PieceKind::ProRook)))
            | (ATTACK_TABLE.ka.attack(to, occ) & (self.piece_kind_bitboard(PieceKind::Bishop) | self.piece_kind_bitboard(PieceKind::ProBishop)))
            | (ATTACK_TABLE.hi.attack(to, occ) & (self.piece_kind_bitboard(PieceKind::Rook) | self.piece_kind_bitboard(PieceKind::ProRook)))
            | (ATTACK_TABLE.ki.attack(to, opp) & (self.piece_kind_bitboard(PieceKind::Gold) | self.piece_kind_bitboard(PieceKind::ProPawn) | self.piece_kind_bitboard(PieceKind::ProLance) | self.piece_kind_bitboard(PieceKind::ProKnight) | self.piece_kind_bitboard(PieceKind::ProSilver) | self.piece_kind_bitboard(PieceKind::ProBishop)))
        ) & self.player_bitboard(c)
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

    #[test]
    fn evasion_moves() {
        // TODO: add more cases
        // Behind RY
        // P1 *  *  *  *  *  *  *  *  *
        // P2 *  *  *  *  *  *  *  *  *
        // P3 *  *  *  *  *  *  *  *  *
        // P4 *  *  *  *  *  *  *  *  *
        // P5 *  *  *  *  *  *  *  *  *
        // P6 *  *  *  *  *  *  * -FU *
        // P7 *  *  *  *  *  *  * -RY *
        // P8 *  *  *  *  *  * +OU+KE *
        // P9 *  *  *  * -OU * +GI *  *
        // P+00FU
        // P-00AL
        // +
        let pos = Position::new(
            PartialPosition::from_usi("sfen 9/9/9/9/9/7p1/7+r1/6KN1/4k1S2 b Pr2b4g3s3n4l16p 1")
                .expect("failed to parse"),
        );
        let moves = pos.legal_moves();
        assert_eq!(1, moves.len());
        assert_eq!(Square::SQ_2I, moves[0].to());
    }

    #[test]
    fn pawn_drop() {
        {
            // P1-KY-KE-GI-KI-OU-KI-GI-KE-KY
            // P2 * -HI *  *  *  *  * -KA *
            // P3-FU-FU-FU-FU-FU-FU-FU-FU *
            // P4 *  *  *  *  *  *  *  *  *
            // P5 *  *  *  *  *  *  *  * +KY
            // P6 *  *  *  *  *  *  *  *  *
            // P7+FU+FU+FU+FU+FU+FU+FU+FU *
            // P8 * +KA *  *  *  *  * +HI *
            // P9+KY+KE+GI+KI+OU+KI+GI+KE *
            // P+00FU
            // P-00FU
            // -
            let pos = Position::new(
                PartialPosition::from_usi(
                    "sfen lnsgkgsnl/1r5s1/pppppppp1/9/8L/9/PPPPPPPP1/1B5S1/LNSGKGSN1 w Pp 1",
                )
                .expect("failed to parse"),
            );
            let drop_moves = pos
                .legal_moves()
                .iter()
                .filter_map(|&m| {
                    if let Move::Drop { piece, to } = m {
                        Some((piece, to))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            assert_eq!(6, drop_moves.len());
            assert!(drop_moves.iter().all(|m| m.0 == Piece::W_P));
        }
        {
            // P1-KY-KE-GI-KI-OU-KI-GI-KE *
            // P2 * -HI *  *  *  *  * -KA *
            // P3-FU-FU-FU-FU-FU-FU-FU-FU *
            // P4 *  *  *  *  *  *  *  *  *
            // P5 *  *  *  *  *  *  *  * -KY
            // P6 *  *  *  *  *  *  *  *  *
            // P7+FU+FU+FU+FU+FU+FU+FU+FU *
            // P8 * +KA *  *  *  *  * +HI *
            // P9+KY+KE+GI+KI+OU+KI+GI+KE *
            // P+00FU
            // P-00FU00KY
            // +
            let pos = Position::new(
                PartialPosition::from_usi(
                    "sfen lnsgkgsn1/1r5s1/pppppppp1/9/8l/9/PPPPPPPP1/1B5S1/LNSGKGSN1 b Ppl 1",
                )
                .expect("failed to parse"),
            );
            let drop_moves = pos
                .legal_moves()
                .iter()
                .filter_map(|&m| {
                    if let Move::Drop { piece, to } = m {
                        Some((piece, to))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            assert_eq!(7, drop_moves.len());
            assert!(drop_moves.iter().all(|m| m.0 == Piece::B_P));
        }
    }

    #[allow(clippy::bool_assert_comparison)]
    #[test]
    fn is_pawn_drop_mate() {
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
            // 香がいても飛で歩を取れる
            // P1 *  *  *  *  *  *  *  * -OU
            // P2 *  *  *  *  *  *  *  *  *
            // P3 *  *  *  *  *  *  * +RY-HI
            // P4 *  *  *  *  *  *  *  * +KY
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
                    PartialPosition::from_usi("sfen 8k/9/7+Rr/8L/9/9/9/9/9 b P2b4g4s4n3l17p 1")
                        .expect("failed to parse"),
                ),
                Square::SQ_1B,
                false,
            ),
        ];
        for (i, (pos, sq, expected)) in test_cases.into_iter().enumerate() {
            assert_eq!(expected, pos.is_pawn_drop_mate(sq), "failed at {i}");
        }
    }
}
