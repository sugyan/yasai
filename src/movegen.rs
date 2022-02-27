use crate::bitboard::Bitboard;
use crate::table::ATTACK_TABLE;
use crate::{Move, PieceType, Position};

#[derive(Default)]
pub struct MoveList(Vec<Move>);

impl MoveList {
    pub fn generate_legals(&mut self, pos: &Position) {
        // TODO: in check
        let color = pos.side_to_move();
        let target = !pos.pieces_c(color);
        self.generate_for_fu(pos, &target);
        self.generate_for_ky(pos, &target);
        self.generate_for_ke(pos, &target);
        self.generate_for_gi(pos, &target);
        self.generate_for_ka(pos, &target);
        self.generate_for_hi(pos, &target);
        self.generate_for_ki(pos, &target);
    }
    fn push(&mut self, m: Move) {
        self.0.push(m);
    }
    fn generate_for_fu(&mut self, pos: &Position, target: &Bitboard) {
        let color = pos.side_to_move();
        for from in pos.pieces_cp(color, PieceType::FU) {
            for to in ATTACK_TABLE.fu.attack(from, color) & *target {
                // TODO: promote?
                self.push(Move::new(from, to, pos.piece_on(from), false));
            }
        }
    }
    fn generate_for_ky(&mut self, pos: &Position, target: &Bitboard) {
        let color = pos.side_to_move();
        for from in pos.pieces_cp(color, PieceType::KY) {
            let occupied = pos.pieces_p(PieceType::OCCUPIED);
            for to in ATTACK_TABLE.ky.attack(from, color, &occupied) & *target {
                // TODO: (force) promote?
                self.push(Move::new(from, to, pos.piece_on(from), false));
            }
        }
    }
    fn generate_for_ke(&mut self, pos: &Position, target: &Bitboard) {
        let color = pos.side_to_move();
        for from in pos.pieces_cp(color, PieceType::KE) {
            for to in ATTACK_TABLE.ke.attack(from, color) & *target {
                // TODO: promote?
                self.push(Move::new(from, to, pos.piece_on(from), false));
            }
        }
    }
    fn generate_for_gi(&mut self, pos: &Position, target: &Bitboard) {
        let color = pos.side_to_move();
        for from in pos.pieces_cp(color, PieceType::GI) {
            for to in ATTACK_TABLE.gi.attack(from, color) & *target {
                // TODO: promote?
                self.push(Move::new(from, to, pos.piece_on(from), false));
            }
        }
    }
    fn generate_for_ka(&mut self, pos: &Position, target: &Bitboard) {
        let color = pos.side_to_move();
        for from in pos.pieces_cp(color, PieceType::KA) {
            let occupied = pos.pieces_p(PieceType::OCCUPIED);
            for to in ATTACK_TABLE.ka.attack(from, &occupied) & *target {
                // TODO: promote?
                self.push(Move::new(from, to, pos.piece_on(from), false));
            }
        }
    }
    fn generate_for_hi(&mut self, pos: &Position, target: &Bitboard) {
        let color = pos.side_to_move();
        for from in pos.pieces_cp(color, PieceType::HI) {
            let occupied = pos.pieces_p(PieceType::OCCUPIED);
            for to in ATTACK_TABLE.hi.attack(from, &occupied) & *target {
                // TODO: promote?
                self.push(Move::new(from, to, pos.piece_on(from), false));
            }
        }
    }
    fn generate_for_ki(&mut self, pos: &Position, target: &Bitboard) {
        let color = pos.side_to_move();
        // TODO: promoted pieces
        for from in pos.pieces_cp(color, PieceType::KI) {
            for to in ATTACK_TABLE.ki.attack(from, color) & *target {
                self.push(Move::new(from, to, pos.piece_on(from), false));
            }
        }
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
