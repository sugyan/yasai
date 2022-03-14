use crate::bitboard::Bitboard;
use crate::square::Rank;
use crate::tables::{ATTACK_TABLE, BETWEEN_TABLE};
use crate::{Color, Move, Piece, PieceType, Position, Square};
use arrayvec::{ArrayVec, IntoIter};
use std::ops::Not;

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
            if pos.is_legal_move(self.0[i]) {
                i += 1;
            } else {
                size -= 1;
                self.0.swap(i, size);
            }
        }
        self.0.truncate(size);
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    fn generate_all(&mut self, pos: &Position) {
        let target = !pos.pieces_c(pos.side_to_move());
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
        self.generate_drop(pos, &(!pos.occupied() & Bitboard::ONES));
    }
    fn generate_evasions(&mut self, pos: &Position) {
        let c = pos.side_to_move();
        if let Some(ou) = pos.king(c) {
            let mut checkers_attacks = Bitboard::ZERO;
            let mut checkers_count = 0;
            for ch in pos.checkers() {
                if let Some(p) = pos.piece_on(ch) {
                    let pt = p.piece_type();
                    // 龍が斜め位置から王手している場合のみ、他の駒の裏に逃がれることができる可能性がある
                    if pt == PieceType::RY && ch.file() != ou.file() && ch.rank() != ou.rank() {
                        checkers_attacks |= ATTACK_TABLE.hi.attack(ch, &pos.occupied());
                    } else {
                        checkers_attacks |= ATTACK_TABLE.pseudo_attack(pt, ch, !c);
                    }
                }
                checkers_count += 1;
            }
            for to in ATTACK_TABLE.ou.attack(ou, c) & !pos.pieces_c(c) & !checkers_attacks {
                self.push(Move::new_normal(
                    ou,
                    to,
                    false,
                    Piece::from_cp(c, PieceType::OU),
                ));
            }
            // 両王手の場合は玉が逃げるしかない
            if checkers_count > 1 {
                return;
            }
            if let Some(ch) = pos.checkers().pop() {
                let target_drop = BETWEEN_TABLE[ch.index()][ou.index()];
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
        let (to_bb, delta, p) = match c {
            Color::Black => (pos.pieces_cp(c, PieceType::FU).shr(), 1, Piece::BFU),
            Color::White => (pos.pieces_cp(c, PieceType::FU).shl(), -1, Piece::WFU),
        };
        for to in to_bb & *target {
            let rank = to.rank();
            let from = Square(to.0 + delta);
            if rank.is_opponent_field(c) {
                self.push(Move::new_normal(from, to, true, p));
                if rank.is_valid_for_piece(c, PieceType::FU) {
                    self.push(Move::new_normal(from, to, false, p));
                }
            } else {
                self.push(Move::new_normal(from, to, false, p));
            }
        }
    }
    fn generate_for_ky(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        let p = Piece::from_cp(c, PieceType::KY);
        for from in pos.pieces_cp(c, PieceType::KY) {
            for to in ATTACK_TABLE.ky.attack(from, c, &pos.occupied()) & *target {
                let rank = to.rank();
                if rank.is_opponent_field(c) {
                    self.push(Move::new_normal(from, to, true, p));
                    if rank.is_valid_for_piece(c, PieceType::KY) {
                        self.push(Move::new_normal(from, to, false, p));
                    }
                } else {
                    self.push(Move::new_normal(from, to, false, p));
                }
            }
        }
    }
    fn generate_for_ke(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        let p = Piece::from_cp(c, PieceType::KE);
        for from in pos.pieces_cp(c, PieceType::KE) {
            for to in ATTACK_TABLE.ke.attack(from, c) & *target {
                let rank = to.rank();
                if rank.is_opponent_field(c) {
                    self.push(Move::new_normal(from, to, true, p));
                    if rank.is_valid_for_piece(c, PieceType::KE) {
                        self.push(Move::new_normal(from, to, false, p));
                    }
                } else {
                    self.push(Move::new_normal(from, to, false, p));
                }
            }
        }
    }
    fn generate_for_gi(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        let p = Piece::from_cp(c, PieceType::GI);
        for from in pos.pieces_cp(c, PieceType::GI) {
            let from_is_opponent_field = from.rank().is_opponent_field(c);
            for to in ATTACK_TABLE.gi.attack(from, c) & *target {
                self.push(Move::new_normal(from, to, false, p));
                if from_is_opponent_field || to.rank().is_opponent_field(c) {
                    self.push(Move::new_normal(from, to, true, p));
                }
            }
        }
    }
    fn generate_for_ka(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        let p = Piece::from_cp(c, PieceType::KA);
        for from in pos.pieces_cp(c, PieceType::KA) {
            let from_is_opponent_field = from.rank().is_opponent_field(c);
            for to in ATTACK_TABLE.ka.attack(from, &pos.occupied()) & *target {
                self.push(Move::new_normal(from, to, false, p));
                if from_is_opponent_field || to.rank().is_opponent_field(c) {
                    self.push(Move::new_normal(from, to, true, p));
                }
            }
        }
    }
    fn generate_for_hi(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        let p = Piece::from_cp(c, PieceType::HI);
        for from in pos.pieces_cp(c, PieceType::HI) {
            let from_is_opponent_field = from.rank().is_opponent_field(c);
            for to in ATTACK_TABLE.hi.attack(from, &pos.occupied()) & *target {
                self.push(Move::new_normal(from, to, false, p));
                if from_is_opponent_field || to.rank().is_opponent_field(c) {
                    self.push(Move::new_normal(from, to, true, p));
                }
            }
        }
    }
    fn generate_for_ki(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        for from in (pos.pieces_p(PieceType::KI)
            | pos.pieces_p(PieceType::TO)
            | pos.pieces_p(PieceType::NY)
            | pos.pieces_p(PieceType::NK)
            | pos.pieces_p(PieceType::NG))
            & pos.pieces_c(c)
        {
            if let Some(p) = pos.piece_on(from) {
                for to in ATTACK_TABLE.ki.attack(from, c) & *target {
                    self.push(Move::new_normal(from, to, false, p));
                }
            }
        }
    }
    fn generate_for_ou(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        let p = Piece::from_cp(c, PieceType::OU);
        for from in pos.pieces_cp(c, PieceType::OU) {
            for to in ATTACK_TABLE.ou.attack(from, c) & *target {
                self.push(Move::new_normal(from, to, false, p));
            }
        }
    }
    fn generate_for_um(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        let p = Piece::from_cp(c, PieceType::UM);
        for from in pos.pieces_cp(c, PieceType::UM) {
            for to in (ATTACK_TABLE.ka.attack(from, &pos.occupied())
                | ATTACK_TABLE.ou.attack(from, c))
                & *target
            {
                self.push(Move::new_normal(from, to, false, p));
            }
        }
    }
    fn generate_for_ry(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        let p = Piece::from_cp(c, PieceType::RY);
        for from in pos.pieces_cp(c, PieceType::RY) {
            for to in (ATTACK_TABLE.hi.attack(from, &pos.occupied())
                | ATTACK_TABLE.ou.attack(from, c))
                & *target
            {
                self.push(Move::new_normal(from, to, false, p));
            }
        }
    }
    fn generate_drop(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        let hand = pos.hand(pos.side_to_move());
        for pt in PieceType::ALL_HAND {
            if hand.num(pt) == 0 {
                continue;
            }
            let mut exclude = Bitboard::ZERO;
            if pt == PieceType::FU {
                // 二歩
                for sq in pos.pieces_cp(c, pt) {
                    exclude |= Bitboard::from_file(sq.file());
                }
                // 打ち歩詰めチェック
                if let Some(sq) = pos.king(!c) {
                    if let Some(to) = ATTACK_TABLE.fu.attack(sq, !c).pop() {
                        if (*target & to).is_empty().not() && Self::is_uchifuzume(pos, to) {
                            exclude |= to;
                        }
                    }
                }
                exclude |= match c {
                    Color::Black => Bitboard::from_rank(Rank::RANK1),
                    Color::White => Bitboard::from_rank(Rank::RANK9),
                };
            }
            for to in *target & !exclude {
                if to.rank().is_valid_for_piece(c, pt) {
                    self.push(Move::new_drop(to, Piece::from_cp(c, pt)));
                }
            }
        }
    }
    fn is_uchifuzume(pos: &Position, sq: Square) -> bool {
        let c = pos.side_to_move();
        // 玉自身が歩を取れる
        if pos.attackers_to(c, sq).is_empty() {
            return false;
        }
        // 他の駒が歩を取れる
        let capture_candidates = Self::attackers_to_except_klp(pos, !c, sq);
        if (capture_candidates & !pos.pinned()[(!c).index()])
            .is_empty()
            .not()
        {
            return false;
        }
        // 玉が逃げられる
        if let Some(king) = pos.king(!c) {
            let escape = ATTACK_TABLE.ou.attack(king, !c) & !pos.pieces_c(!c);
            for to in escape ^ sq {
                if pos.attackers_to(c, to).is_empty() {
                    return false;
                }
            }
        }
        true
    }
    #[rustfmt::skip]
    fn attackers_to_except_klp(pos: &Position, c: Color, to: Square) -> Bitboard {
        let opp = !c;
        let occ = &pos.occupied();
        (     (ATTACK_TABLE.ke.attack(to, opp) & pos.pieces_p(PieceType::KE))
            | (ATTACK_TABLE.gi.attack(to, opp) & (pos.pieces_p(PieceType::GI) | pos.pieces_p(PieceType::RY)))
            | (ATTACK_TABLE.ka.attack(to, occ) & (pos.pieces_p(PieceType::KA) | pos.pieces_p(PieceType::UM)))
            | (ATTACK_TABLE.hi.attack(to, occ) & (pos.pieces_p(PieceType::HI) | pos.pieces_p(PieceType::RY)))
            | (ATTACK_TABLE.ki.attack(to, opp) & (pos.pieces_p(PieceType::KI) | pos.pieces_p(PieceType::TO) | pos.pieces_p(PieceType::NY) | pos.pieces_p(PieceType::NK) | pos.pieces_p(PieceType::NG) | pos.pieces_p(PieceType::UM)))
        ) & pos.pieces_c(c)
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
    use crate::board_piece::*;
    use crate::Color;

    #[test]
    fn from_default() {
        let pos = Position::default();
        assert_eq!(30, pos.legal_moves().len());
    }

    #[test]
    fn drop_moves() {
        #[rustfmt::skip]
        let pos = Position::new([
            WKY, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BKY,
            WKE, WGI, WFU, EMP, EMP, EMP, BFU, BHI, BKE,
            EMP, EMP, EMP, WFU, EMP, EMP, BFU, EMP, BGI,
            WKI, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BKI,
            WOU, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BOU,
            WKI, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BKI,
            WGI, EMP, WFU, EMP, EMP, BFU, EMP, EMP, BGI,
            WKE, WHI, WFU, EMP, EMP, EMP, BFU, EMP, BKE,
            WKY, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BKY,
        ], [
            [0, 0, 0, 0, 0, 1, 0],
            [0, 0, 0, 0, 0, 1, 0],
        ], Color::Black, 1);
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
        // R8/2K1S1SSk/4B4/9/9/9/9/9/1L1L1L3 b RBGSNLP3g3n17p 1
        #[rustfmt::skip]
        let pos = Position::new([
            EMP, WOU, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
            EMP, BGI, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
            EMP, BGI, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, BKY,
            EMP, BGI, BKA, EMP, EMP, EMP, EMP, EMP, EMP,
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, BKY,
            EMP, BOU, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, BKY,
            BHI, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
        ], [
            [ 1, 1, 1, 1, 1, 1, 1],
            [17, 0, 3, 0, 3, 0, 0],
        ], Color::Black, 1);
        assert_eq!(593, pos.legal_moves().len());
    }

    #[allow(clippy::bool_assert_comparison)]
    #[test]
    fn is_uchifuzume() {
        #[rustfmt::skip]
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
                Position::new([
                    EMP, WFU, WOU, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, WFU, EMP, BFU, BKI, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                ], [
                    [ 1, 0, 0, 0, 0, 0, 0],
                    [14, 4, 4, 4, 3, 2, 2],
                ], Color::Black, 1),
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
                Position::new([
                    EMP, WFU, WOU, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, WFU, WKI, EMP, BKI, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                ], [
                    [ 1, 0, 0, 0, 0, 0, 0],
                    [15, 4, 4, 4, 2, 2, 2],
                ], Color::Black, 1),
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
                Position::new([
                    EMP, WFU, WOU, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, WFU, WKI, BFU, BKI, EMP, EMP, EMP, EMP,
                    EMP, EMP, BHI, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                ], [
                    [ 1, 0, 0, 0, 0, 0, 0],
                    [15, 4, 4, 4, 2, 2, 1],
                ], Color::Black, 1),
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
                Position::new([
                    EMP, WFU, WOU, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, WKE, EMP, BFU, BKI, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                ], [
                    [ 1, 0, 0, 0, 0, 0, 0],
                    [15, 4, 3, 4, 3, 2, 2],
                ], Color::Black, 1),
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
                Position::new([
                    EMP, WFU, WOU, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, WKE, EMP, BFU, BKI, EMP, EMP, EMP, EMP,
                    BKA, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                ], [
                    [ 1, 0, 0, 0, 0, 0, 0],
                    [15, 4, 3, 4, 3, 1, 2],
                ], Color::Black, 1),
                true,
            ),
        ];
        for (pos, expected) in test_cases {
            assert_eq!(
                expected,
                MoveList::is_uchifuzume(&pos, Square::SQ14),
                "\n{pos}"
            );
        }
    }
}
