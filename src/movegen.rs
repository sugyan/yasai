use crate::bitboard::Bitboard;
use crate::square::Rank;
use crate::tables::{ATTACK_TABLE, BETWEEN_TABLE};
use crate::{Color, Move, Piece, PieceType, Position};
use arrayvec::ArrayVec;

pub struct MoveList(ArrayVec<Move, { MoveList::MAX_LEGAL_MOVES }>);

impl MoveList {
    const MAX_LEGAL_MOVES: usize = 593;

    pub fn generate_legals(&mut self, pos: &Position) {
        if pos.in_check() {
            self.generate_evasions(pos);
        } else {
            self.generate_all(pos);
        }
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    fn generate_all(&mut self, pos: &Position) {
        let target = !pos.pieces_c(pos.side_to_move());
        for &pt in PieceType::ALL.iter().skip(1) {
            self.generate_for_piece(pt, pos, &target);
        }
        self.generate_drop(pos, &(!pos.occupied() & Bitboard::ONES));
    }
    fn generate_evasions(&mut self, pos: &Position) {
        let c = pos.side_to_move();
        if let Some(ou) = pos.king(c) {
            let mut checkers_attacks = Bitboard::ZERO;
            let mut checkers_count = 0;
            for ch in pos.checkers() {
                if let Some(pt) = pos.piece_on(ch).piece_type() {
                    checkers_attacks |= ATTACK_TABLE.pseudo_attack(pt, ch, c);
                }
                checkers_count += 1;
            }
            for to in ATTACK_TABLE.attack(PieceType::OU, ou, !c, &Bitboard::ZERO)
                & !pos.pieces_c(c)
                & !checkers_attacks
            {
                self.push(
                    Move::new_normal(ou, to, false, pos.piece_on(ou), pos.piece_on(to)),
                    pos,
                );
            }
            // 両王手の場合は玉が逃げるしかない
            if checkers_count > 1 {
                return;
            }
            if let Some(ch) = pos.checkers().pop() {
                let target_drop = BETWEEN_TABLE[ch.index()][ou.index()];
                let target_move = target_drop | pos.checkers();
                for &pt in PieceType::ALL.iter().skip(1) {
                    if pt != PieceType::OU {
                        self.generate_for_piece(pt, pos, &target_move);
                    }
                }
                self.generate_drop(pos, &target_drop);
            }
        }
    }
    fn push(&mut self, m: Move, pos: &Position) {
        if let Some(from) = m.from() {
            let c = pos.side_to_move();
            // 玉が相手の攻撃範囲内に動いてしまう指し手は除外
            if pos.piece_on(from).piece_type() == Some(PieceType::OU)
                && !pos.attackers_to(!c, m.to()).is_empty()
            {
                return;
            }
            // 飛び駒から守っている駒が直線上から外れてしまう指し手は除外
            if !(pos.pinned() & from).is_empty() {
                if let Some(sq) = pos.king(c) {
                    if (BETWEEN_TABLE[sq.index()][from.index()] & m.to()).is_empty()
                        && (BETWEEN_TABLE[sq.index()][m.to().index()] & from).is_empty()
                    {
                        return;
                    }
                }
            }
        }
        self.0.push(m);
    }
    fn generate_for_piece(&mut self, pt: PieceType, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        let occ = pos.occupied();
        for from in pos.pieces_cp(c, pt) {
            let p_from = pos.piece_on(from);
            let from_is_opponent_field = from.rank().is_opponent_field(c);
            for to in ATTACK_TABLE.attack(pt, from, c, &occ) & *target {
                if to.rank().is_valid_for_piece(c, pt) {
                    self.push(
                        Move::new_normal(from, to, false, p_from, pos.piece_on(to)),
                        pos,
                    );
                }
                if pt.is_promotable() && (from_is_opponent_field || to.rank().is_opponent_field(c))
                {
                    self.push(
                        Move::new_normal(from, to, true, p_from.promoted(), pos.piece_on(to)),
                        pos,
                    );
                }
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
                // 二歩回避
                for sq in pos.pieces_cp(c, pt) {
                    exclude |= Bitboard::from_file(sq.file());
                }
                exclude |= match c {
                    Color::Black => Bitboard::from_rank(Rank::RANK1),
                    Color::White => Bitboard::from_rank(Rank::RANK9),
                };
            }
            for to in *target & !exclude {
                if to.rank().is_valid_for_piece(c, pt) {
                    if let Some(p) = Piece::from_cp(c, pt) {
                        self.push(Move::new_drop(to, p), pos);
                    }
                }
            }
        }
    }
}

impl Default for MoveList {
    fn default() -> Self {
        Self(ArrayVec::new())
    }
}

impl std::ops::Index<usize> for MoveList {
    type Output = Move;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IntoIterator for MoveList {
    type Item = Move;
    type IntoIter = MoveListIter;
    fn into_iter(self) -> Self::IntoIter {
        MoveListIter {
            moves: self,
            index: 0,
        }
    }
}

pub struct MoveListIter {
    moves: MoveList,
    index: usize,
}

impl Iterator for MoveListIter {
    type Item = Move;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.moves.0.len() {
            let m = self.moves[self.index];
            self.index += 1;
            Some(m)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, Piece};

    #[test]
    fn test_from_default() {
        let pos = Position::default();
        assert_eq!(30, pos.legal_moves().len());
    }

    #[test]
    fn test_drop_moves() {
        #[rustfmt::skip]
        let pos = Position::new([
            Piece::WKY, Piece::EMP, Piece::WFU, Piece::EMP, Piece::EMP, Piece::EMP, Piece::BFU, Piece::EMP, Piece::BKY,
            Piece::WKE, Piece::WGI, Piece::WFU, Piece::EMP, Piece::EMP, Piece::EMP, Piece::BFU, Piece::BHI, Piece::BKE,
            Piece::EMP, Piece::EMP, Piece::EMP, Piece::WFU, Piece::EMP, Piece::EMP, Piece::BFU, Piece::EMP, Piece::BGI,
            Piece::WKI, Piece::EMP, Piece::WFU, Piece::EMP, Piece::EMP, Piece::EMP, Piece::BFU, Piece::EMP, Piece::BKI,
            Piece::WOU, Piece::EMP, Piece::WFU, Piece::EMP, Piece::EMP, Piece::EMP, Piece::BFU, Piece::EMP, Piece::BOU,
            Piece::WKI, Piece::EMP, Piece::WFU, Piece::EMP, Piece::EMP, Piece::EMP, Piece::BFU, Piece::EMP, Piece::BKI,
            Piece::WGI, Piece::EMP, Piece::WFU, Piece::EMP, Piece::EMP, Piece::BFU, Piece::EMP, Piece::EMP, Piece::BGI,
            Piece::WKE, Piece::WHI, Piece::WFU, Piece::EMP, Piece::EMP, Piece::EMP, Piece::BFU, Piece::EMP, Piece::BKE,
            Piece::WKY, Piece::EMP, Piece::WFU, Piece::EMP, Piece::EMP, Piece::EMP, Piece::BFU, Piece::EMP, Piece::BKY,
        ], [
            [0, 0, 0, 0, 0, 1, 0],
            [0, 0, 0, 0, 0, 1, 0],
        ], Color::Black);
        assert_eq!(
            43,
            pos.legal_moves()
                .into_iter()
                .filter(|m| m.is_drop())
                .count()
        );
    }

    #[test]
    fn test_maximum_moves() {
        // R8/2K1S1SSk/4B4/9/9/9/9/9/1L1L1L3 b RBGSNLP3g3n17p 1
        #[rustfmt::skip]
        let pos = Position::new([
            Piece::EMP, Piece::WOU, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP,
            Piece::EMP, Piece::BGI, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP,
            Piece::EMP, Piece::BGI, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP,
            Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::BKY,
            Piece::EMP, Piece::BGI, Piece::BKA, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP,
            Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::BKY,
            Piece::EMP, Piece::BOU, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP,
            Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::BKY,
            Piece::BHI, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP,
        ], [
            [ 1, 1, 1, 1, 1, 1, 1],
            [17, 0, 3, 0, 3, 0, 0],
        ], Color::Black);
        assert_eq!(593, pos.legal_moves().len());
    }
}
