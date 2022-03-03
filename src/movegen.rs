use crate::bitboard::Bitboard;
use crate::tables::{ATTACK_TABLE, BETWEEN_TABLE};
use crate::{Move, PieceType, Position};
use PieceType::*;

pub struct MoveList(Vec<Move>);

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
    }
    fn generate_evasions(&mut self, pos: &Position) {
        let c = pos.side_to_move();
        if let Some(sq) = pos.king(c) {
            let mut checkers_attacks = Bitboard::ZERO;
            let mut checkers_count = 0;
            for ch in pos.checkers() {
                if let Some(pt) = pos.piece_on(ch).piece_type() {
                    checkers_attacks |= ATTACK_TABLE.pseudo_attack(pt, ch, c);
                }
                checkers_count += 1;
            }
            for to in ATTACK_TABLE.attack(OU, sq, !c, &Bitboard::ZERO)
                & !pos.pieces_c(c)
                & !checkers_attacks
            {
                self.push(Move::new_normal(sq, to, false, pos.piece_on(sq)), pos);
            }
            // 両王手の場合は玉が逃げるしかない
            if checkers_count > 1 {
                return;
            }
            if let Some(ch) = pos.checkers().pop() {
                let target = pos.checkers() | BETWEEN_TABLE[ch.index()][sq.index()];
                for &pt in PieceType::ALL.iter().skip(1) {
                    if pt != PieceType::OU {
                        self.generate_for_piece(pt, pos, &target);
                    }
                }
                // TODO: drop
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
                self.push(Move::new_normal(from, to, false, p_from), pos);
                if pt.is_promotable() && (from_is_opponent_field || to.rank().is_opponent_field(c))
                {
                    self.push(Move::new_normal(from, to, true, p_from), pos);
                }
            }
        }
    }
}

impl Default for MoveList {
    fn default() -> Self {
        Self(Vec::with_capacity(MoveList::MAX_LEGAL_MOVES))
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

    #[test]
    fn test_from_default() {
        let pos = Position::default();
        assert_eq!(30, pos.legal_moves().len());
    }
}
