use crate::tables::{ATTACK_TABLE, BETWEEN_TABLE, FILES, PROMOTABLE, RANKS, RELATIVE_RANKS};
use crate::{Move, Position};
use arrayvec::{ArrayVec, IntoIter};
use shogi_core::{Bitboard, Color, Piece, PieceKind, Square};

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
        self.generate_drop(pos, &(!pos.occupied() & !Bitboard::empty()));
    }
    fn generate_evasions(&mut self, pos: &Position) {
        let c = pos.side_to_move();
        if let Some(ou) = pos.king(c) {
            let mut checkers_attacks = Bitboard::empty();
            let mut checkers_count = 0;
            for ch in pos.checkers() {
                if let Some(p) = pos.piece_on(ch) {
                    let pk = p.piece_kind();
                    // 龍が斜め位置から王手している場合のみ、他の駒の裏に逃がれることができる可能性がある
                    if pk == PieceKind::ProRook && ch.file() != ou.file() && ch.rank() != ou.rank()
                    {
                        checkers_attacks |= ATTACK_TABLE.hi.attack(ch, &pos.occupied());
                    } else {
                        checkers_attacks |= ATTACK_TABLE.pseudo_attack(pk, ch, c.flip());
                    }
                }
                checkers_count += 1;
            }
            for to in ATTACK_TABLE.ou.attack(ou, c) & !pos.pieces_c(c) & !checkers_attacks {
                self.push(Move::new_normal(
                    ou,
                    to,
                    false,
                    Piece::new(PieceKind::King, c),
                ));
            }
            // 両王手の場合は玉が逃げるしかない
            if checkers_count > 1 {
                return;
            }
            if let Some(ch) = pos.checkers().pop() {
                let target_drop = BETWEEN_TABLE[ch.array_index()][ou.array_index()];
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
            Color::Black => (
                unsafe { pos.pieces_cp(c, PieceKind::Pawn).shift_up(1) },
                1,
                Piece::B_P,
            ),
            Color::White => (
                unsafe { pos.pieces_cp(c, PieceKind::Pawn).shift_down(1) },
                !0,
                Piece::W_P,
            ),
        };
        for to in to_bb & *target {
            let from = unsafe { Square::from_u8_unchecked(to.index().wrapping_add(delta)) };
            if PROMOTABLE[to.array_index()][c.array_index()] {
                self.push(Move::new_normal(from, to, true, p));
                if RELATIVE_RANKS[to.array_index()][c.array_index()] > 1 {
                    self.push(Move::new_normal(from, to, false, p));
                }
            } else {
                self.push(Move::new_normal(from, to, false, p));
            }
        }
    }
    fn generate_for_ky(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        let p = match c {
            Color::Black => Piece::B_L,
            Color::White => Piece::W_L,
        };
        for from in pos.pieces_cp(c, PieceKind::Lance) {
            for to in ATTACK_TABLE.ky.attack(from, c, &pos.occupied()) & *target {
                if PROMOTABLE[to.array_index()][c.array_index()] {
                    self.push(Move::new_normal(from, to, true, p));
                    if RELATIVE_RANKS[to.array_index()][c.array_index()] > 1 {
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
        let p = match c {
            Color::Black => Piece::B_N,
            Color::White => Piece::W_N,
        };
        for from in pos.pieces_cp(c, PieceKind::Knight) {
            for to in ATTACK_TABLE.ke.attack(from, c) & *target {
                if PROMOTABLE[to.array_index()][c.array_index()] {
                    self.push(Move::new_normal(from, to, true, p));
                    if RELATIVE_RANKS[to.array_index()][c.array_index()] > 2 {
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
        let p = match c {
            Color::Black => Piece::B_S,
            Color::White => Piece::W_S,
        };
        for from in pos.pieces_cp(c, PieceKind::Silver) {
            let from_is_opponent_field = PROMOTABLE[from.array_index()][c.array_index()];
            for to in ATTACK_TABLE.gi.attack(from, c) & *target {
                self.push(Move::new_normal(from, to, false, p));
                if from_is_opponent_field || PROMOTABLE[to.array_index()][c.array_index()] {
                    self.push(Move::new_normal(from, to, true, p));
                }
            }
        }
    }
    fn generate_for_ka(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        let p = match c {
            Color::Black => Piece::B_B,
            Color::White => Piece::W_B,
        };
        for from in pos.pieces_cp(c, PieceKind::Bishop) {
            let from_is_opponent_field = PROMOTABLE[from.array_index()][c.array_index()];
            for to in ATTACK_TABLE.ka.attack(from, &pos.occupied()) & *target {
                self.push(Move::new_normal(from, to, false, p));
                if from_is_opponent_field || PROMOTABLE[to.array_index()][c.array_index()] {
                    self.push(Move::new_normal(from, to, true, p));
                }
            }
        }
    }
    fn generate_for_hi(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        let p = match c {
            Color::Black => Piece::B_R,
            Color::White => Piece::W_R,
        };
        for from in pos.pieces_cp(c, PieceKind::Rook) {
            let from_is_opponent_field = PROMOTABLE[from.array_index()][c.array_index()];
            for to in ATTACK_TABLE.hi.attack(from, &pos.occupied()) & *target {
                self.push(Move::new_normal(from, to, false, p));
                if from_is_opponent_field || PROMOTABLE[to.array_index()][c.array_index()] {
                    self.push(Move::new_normal(from, to, true, p));
                }
            }
        }
    }
    fn generate_for_ki(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        for from in (pos.pieces_p(PieceKind::Gold)
            | pos.pieces_p(PieceKind::ProPawn)
            | pos.pieces_p(PieceKind::ProLance)
            | pos.pieces_p(PieceKind::ProKnight)
            | pos.pieces_p(PieceKind::ProSilver))
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
        let p = match c {
            Color::Black => Piece::B_K,
            Color::White => Piece::W_K,
        };
        for from in pos.pieces_cp(c, PieceKind::King) {
            for to in ATTACK_TABLE.ou.attack(from, c) & *target {
                self.push(Move::new_normal(from, to, false, p));
            }
        }
    }
    fn generate_for_um(&mut self, pos: &Position, target: &Bitboard) {
        let c = pos.side_to_move();
        let p = match c {
            Color::Black => Piece::B_PB,
            Color::White => Piece::W_PB,
        };
        for from in pos.pieces_cp(c, PieceKind::ProBishop) {
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
        let p = match c {
            Color::Black => Piece::B_PR,
            Color::White => Piece::W_PR,
        };
        for from in pos.pieces_cp(c, PieceKind::ProRook) {
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
        for pk in PieceKind::all() {
            if hand.count(pk).unwrap_or_default() == 0 {
                continue;
            }
            let mut exclude = Bitboard::empty();
            if pk == PieceKind::Pawn {
                // 二歩
                for sq in pos.pieces_cp(c, pk) {
                    exclude |= FILES[sq.file() as usize];
                }
                // 打ち歩詰めチェック
                if let Some(sq) = pos.king(c.flip()) {
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
                    self.push(Move::new_drop(to, Piece::new(pk, c)));
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
            if pos.piece_on(from) == Some(king)
                && !pos
                    .attackers_to(c.flip(), m.to(), &pos.occupied())
                    .is_empty()
            {
                return false;
            }
            // 飛び駒から守っている駒が直線上から外れてしまう指し手は除外
            if pos.pinned()[c.array_index()].contains(from) {
                if let Some(sq) = pos.king(c) {
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
        if !(capture_candidates & !pos.pinned()[c.flip().array_index()]).is_empty() {
            return false;
        }
        // 玉が逃げられる
        if let Some(king) = pos.king(c.flip()) {
            let escape = ATTACK_TABLE.ou.attack(king, c.flip())
                & !pos.pieces_c(c.flip())
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
        (     (ATTACK_TABLE.ke.attack(to, opp) & pos.pieces_p(PieceKind::Knight))
            | (ATTACK_TABLE.gi.attack(to, opp) & (pos.pieces_p(PieceKind::Silver) | pos.pieces_p(PieceKind::ProRook)))
            | (ATTACK_TABLE.ka.attack(to, occ) & (pos.pieces_p(PieceKind::Bishop) | pos.pieces_p(PieceKind::ProBishop)))
            | (ATTACK_TABLE.hi.attack(to, occ) & (pos.pieces_p(PieceKind::Rook) | pos.pieces_p(PieceKind::ProRook)))
            | (ATTACK_TABLE.ki.attack(to, opp) & (pos.pieces_p(PieceKind::Gold) | pos.pieces_p(PieceKind::ProPawn) | pos.pieces_p(PieceKind::ProLance) | pos.pieces_p(PieceKind::ProKnight) | pos.pieces_p(PieceKind::ProSilver) | pos.pieces_p(PieceKind::ProBishop)))
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
            [0, 0, 0, 0, 0, 1, 0, 0],
            [0, 0, 0, 0, 0, 1, 0, 0],
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
            [ 1, 1, 1, 1, 1, 1, 1, 0],
            [17, 0, 3, 0, 3, 0, 0, 0],
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
                    [ 1, 0, 0, 0, 0, 0, 0, 0],
                    [14, 4, 4, 4, 3, 2, 2, 0],
                ], Color::Black, 1),
                Square::SQ_1D, true,
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
                    [ 1, 0, 0, 0, 0, 0, 0, 0],
                    [15, 4, 4, 4, 2, 2, 2, 0],
                ], Color::Black, 1),
                Square::SQ_1D, false,
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
                    [ 1, 0, 0, 0, 0, 0, 0, 0],
                    [15, 4, 4, 4, 2, 2, 1, 0],
                ], Color::Black, 1),
                Square::SQ_1D, true,
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
                    [ 1, 0, 0, 0, 0, 0, 0, 0],
                    [15, 4, 3, 4, 3, 2, 2, 0],
                ], Color::Black, 1),
                Square::SQ_1D, false,
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
                    [ 1, 0, 0, 0, 0, 0, 0, 0],
                    [15, 4, 3, 4, 3, 1, 2, 0],
                ], Color::Black, 1),
                Square::SQ_1D, true,
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
                Position::new([
                    EMP, WKY, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    WOU, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, BKA, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, BKI, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                ], [
                    [ 1, 0, 0, 0, 0, 0, 0, 0],
                    [15, 4, 3, 4, 3, 1, 2, 0],
                ], Color::Black, 1),
                Square::SQ_2B, false,
            ),
        ];
        for (pos, sq, expected) in test_cases {
            assert_eq!(expected, MoveList::is_uchifuzume(&pos, sq));
        }
    }
}
