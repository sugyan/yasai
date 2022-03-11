use crate::bitboard::Bitboard;
use crate::hand::Hand;
use crate::movegen::MoveList;
use crate::piece::PieceType;
use crate::square::{File, Rank};
use crate::tables::{ATTACK_TABLE, BETWEEN_TABLE};
use crate::{Color, Move, Piece, Square};
use std::fmt;
use std::ops::Not;

#[derive(Debug)]
struct AttackInfo {
    checkers: Bitboard,                     // 王手をかけている駒の位置
    checkables: [Bitboard; PieceType::NUM], // 各駒種が王手になり得る位置
    pinned: [Bitboard; Color::NUM],         // 飛び駒から玉を守っている駒の位置
}

impl AttackInfo {
    fn new(checkers: Bitboard, c_bb: &[Bitboard], pt_bb: &[Bitboard], c: Color) -> Self {
        let mut pinned = [Bitboard::ZERO, Bitboard::ZERO];
        let occ = pt_bb[PieceType::OCCUPIED.index()];
        for c in Color::ALL {
            if let Some(sq) = (c_bb[c.index()] & pt_bb[PieceType::OU.index()]).pop() {
                #[rustfmt::skip]
                let snipers = (
                      (ATTACK_TABLE.pseudo_attack(PieceType::KY, sq, c) & pt_bb[PieceType::KY.index()])
                    | (ATTACK_TABLE.pseudo_attack(PieceType::KA, sq, c) & (pt_bb[PieceType::KA.index()] | pt_bb[PieceType::UM.index()]))
                    | (ATTACK_TABLE.pseudo_attack(PieceType::HI, sq, c) & (pt_bb[PieceType::HI.index()] | pt_bb[PieceType::RY.index()]))
                ) & c_bb[(!c).index()];
                for sniper in snipers {
                    let blockers = BETWEEN_TABLE[sq.index()][sniper.index()] & occ;
                    if blockers.count_ones() == 1 {
                        pinned[c.index()] |= blockers;
                    }
                }
            }
        }
        if let Some(sq) = (c_bb[(!c).index()] & pt_bb[PieceType::OU.index()]).pop() {
            let ka = ATTACK_TABLE.ka.attack(sq, &occ);
            let hi = ATTACK_TABLE.hi.attack(sq, &occ);
            let ki = ATTACK_TABLE.ki.attack(sq, !c);
            let ou = ATTACK_TABLE.ou.attack(sq, !c);
            Self {
                checkers,
                checkables: [
                    Bitboard::ZERO,
                    ATTACK_TABLE.fu.attack(sq, !c),
                    ATTACK_TABLE.ky.attack(sq, !c, &occ),
                    ATTACK_TABLE.ke.attack(sq, !c),
                    ATTACK_TABLE.gi.attack(sq, !c),
                    ka,
                    hi,
                    ki,
                    Bitboard::ZERO,
                    ki,
                    ki,
                    ki,
                    ki,
                    ka | ou,
                    hi | ou,
                ],
                pinned,
            }
        } else {
            Self {
                checkers,
                checkables: [Bitboard::ZERO; PieceType::NUM],
                pinned,
            }
        }
    }
    #[rustfmt::skip]
    fn calculate_checkers(c_bb: &[Bitboard], pt_bb: &[Bitboard], c: Color) -> Bitboard {
        let opp = !c;
        if let Some(sq) = (c_bb[opp.index()] & pt_bb[PieceType::OU.index()]).pop() {
            let occ = &pt_bb[PieceType::OCCUPIED.index()];
            (     (ATTACK_TABLE.fu.attack(sq, opp)      & pt_bb[PieceType::FU.index()])
                | (ATTACK_TABLE.ky.attack(sq, opp, occ) & pt_bb[PieceType::KY.index()])
                | (ATTACK_TABLE.ke.attack(sq, opp)      & pt_bb[PieceType::KE.index()])
                | (ATTACK_TABLE.gi.attack(sq, opp)      & (pt_bb[PieceType::GI.index()] | pt_bb[PieceType::RY.index()]))
                | (ATTACK_TABLE.ka.attack(sq, occ)      & (pt_bb[PieceType::KA.index()] | pt_bb[PieceType::UM.index()]))
                | (ATTACK_TABLE.hi.attack(sq, occ)      & (pt_bb[PieceType::HI.index()] | pt_bb[PieceType::RY.index()]))
                | (ATTACK_TABLE.ki.attack(sq, opp)      & (pt_bb[PieceType::KI.index()] | pt_bb[PieceType::TO.index()] | pt_bb[PieceType::NY.index()] | pt_bb[PieceType::NK.index()] | pt_bb[PieceType::NG.index()] | pt_bb[PieceType::UM.index()]))
            ) & c_bb[c.index()]
        } else {
            Bitboard::ZERO
        }
    }
}

#[derive(Debug)]
struct State {
    captured: Piece,
    attack_info: AttackInfo,
}

impl State {
    fn new(captured: Piece, attack_info: AttackInfo) -> Self {
        Self {
            captured,
            attack_info,
        }
    }
}

/// Represents a state of the game.
#[derive(Debug)]
pub struct Position {
    board: [Piece; Square::NUM],
    hands: [Hand; Color::NUM],
    color: Color,
    c_bb: [Bitboard; Color::NUM],
    pt_bb: [Bitboard; PieceType::NUM],
    states: Vec<State>,
}

impl Position {
    pub fn new(
        board: [Piece; Square::NUM],
        hand_nums: [[u8; PieceType::NUM_HAND]; Color::NUM],
        side_to_move: Color,
    ) -> Position {
        // board
        let mut c_bb = [Bitboard::ZERO; Color::NUM];
        let mut pt_bb = [Bitboard::ZERO; PieceType::NUM];
        for sq in Square::ALL {
            let piece = board[sq.index()];
            if let Some(c) = piece.color() {
                c_bb[c.index()] |= sq;
            }
            if let Some(pt) = piece.piece_type() {
                pt_bb[PieceType::OCCUPIED.index()] |= sq;
                pt_bb[pt.index()] |= sq;
            }
        }
        // hands
        let mut hands = [Hand::new(); Color::NUM];
        for c in Color::ALL {
            for (i, &num) in hand_nums[c.index()].iter().enumerate() {
                for _ in 0..num {
                    hands[c.index()].increment(PieceType::ALL_HAND[i]);
                }
            }
        }
        // initial state
        let checkers = AttackInfo::calculate_checkers(&c_bb, &pt_bb, side_to_move);
        Self {
            board,
            hands,
            color: side_to_move,
            pt_bb,
            c_bb,
            states: vec![State::new(
                Piece::EMP,
                AttackInfo::new(checkers, &c_bb, &pt_bb, side_to_move),
            )],
        }
    }
    pub fn piece_on(&self, sq: Square) -> Piece {
        self.board[sq.index()]
    }
    pub fn pieces_cp(&self, c: Color, pt: PieceType) -> Bitboard {
        self.pieces_c(c) & self.pieces_p(pt)
    }
    pub fn pieces_c(&self, c: Color) -> Bitboard {
        self.c_bb[c.index()]
    }
    pub fn pieces_p(&self, pt: PieceType) -> Bitboard {
        self.pt_bb[pt.index()]
    }
    pub fn pieces_ps(&self, pts: &[PieceType]) -> Bitboard {
        pts.iter()
            .fold(Bitboard::ZERO, |acc, &pt| acc | self.pieces_p(pt))
    }
    pub fn occupied(&self) -> Bitboard {
        self.pt_bb[PieceType::OCCUPIED.index()]
    }
    pub fn in_check(&self) -> bool {
        self.checkers().is_empty().not()
    }
    pub fn captured(&self) -> Piece {
        self.state().captured
    }
    pub fn checkers(&self) -> Bitboard {
        self.state().attack_info.checkers
    }
    fn checkable(&self, pt: PieceType, sq: Square) -> bool {
        (self.state().attack_info.checkables[pt.index()] & sq)
            .is_empty()
            .not()
    }
    pub fn pinned(&self) -> [Bitboard; Color::NUM] {
        self.state().attack_info.pinned
    }
    pub fn king(&self, c: Color) -> Option<Square> {
        self.pieces_cp(c, PieceType::OU).pop()
    }
    pub fn hand(&self, c: Color) -> Hand {
        self.hands[c.index()]
    }
    pub fn side_to_move(&self) -> Color {
        self.color
    }
    pub fn legal_moves(&self) -> MoveList {
        let mut ml = MoveList::default();
        ml.generate_legals(self);
        ml
    }
    pub fn do_move(&mut self, m: Move) {
        let is_check = self.is_check_move(m);
        let c = self.side_to_move();
        let to = m.to();
        // 駒移動
        if let Some(from) = m.from() {
            let p_from = self.piece_on(from);
            self.remove_piece(from, p_from);
            // 移動先に駒がある場合
            let p_cap = self.piece_on(to);
            if let Some(pt) = p_cap.piece_type() {
                self.xor_bbs(!c, pt, to);
                self.hands[c.index()].increment(pt);
            }
            let p_to = if m.is_promotion() {
                p_from.promoted()
            } else {
                p_from
            };
            self.put_piece(to, p_to);
            let checkers = if is_check {
                AttackInfo::calculate_checkers(&self.c_bb, &self.pt_bb, c)
            } else {
                Bitboard::ZERO
            };
            self.states.push(State::new(
                p_cap,
                AttackInfo::new(checkers, &self.c_bb, &self.pt_bb, !c),
            ));
        }
        // 駒打ち
        else {
            let p = m.piece();
            let pt = p.piece_type().expect("empty piece for drop move");
            self.put_piece(to, p);
            self.hands[c.index()].decrement(pt);
            let checkers = if is_check {
                Bitboard::from_square(to)
            } else {
                Bitboard::ZERO
            };
            self.states.push(State::new(
                Piece::EMP,
                AttackInfo::new(checkers, &self.c_bb, &self.pt_bb, !c),
            ));
        };
        self.color = !self.color;
    }
    pub fn undo_move(&mut self, m: Move) {
        let c = self.side_to_move();
        let to = m.to();
        let p_to = self.piece_on(to);
        // 駒移動
        if let Some(from) = m.from() {
            self.remove_piece(to, p_to);
            let p_cap = self.captured();
            if let Some(pt) = p_cap.piece_type() {
                self.put_piece(to, p_cap);
                self.hands[(!c).index()].decrement(pt);
            }
            let p_from = if m.is_promotion() {
                p_to.demoted()
            } else {
                p_to
            };
            self.put_piece(from, p_from);
        }
        // 駒打ち
        else {
            self.remove_piece(to, p_to);
            if let Some(pt) = p_to.piece_type() {
                self.hands[(!c).index()].increment(pt);
            }
        }
        self.color = !self.color;
        self.states.pop();
    }
    fn state(&self) -> &State {
        self.states.last().expect("empty states")
    }
    fn put_piece(&mut self, sq: Square, p: Piece) {
        if let (Some(c), Some(pt)) = (p.color(), p.piece_type()) {
            self.xor_bbs(c, pt, sq);
        } else {
            panic!("failed to put piece: square: {:?}, piece: {:?}", sq, p);
        }
        self.board[sq.index()] = p;
    }
    fn remove_piece(&mut self, sq: Square, p: Piece) {
        if let (Some(c), Some(pt)) = (p.color(), p.piece_type()) {
            self.xor_bbs(c, pt, sq);
        } else {
            panic!("failed to remove piece: square: {:?}, piece: {:?}", sq, p);
        }
        self.board[sq.index()] = Piece::EMP;
    }
    fn xor_bbs(&mut self, c: Color, pt: PieceType, sq: Square) {
        self.c_bb[c.index()] ^= sq;
        self.pt_bb[PieceType::OCCUPIED.index()] ^= sq;
        self.pt_bb[pt.index()] ^= sq;
    }
    #[rustfmt::skip]
    pub fn attackers_to(&self, c: Color, to: Square) -> Bitboard {
        let opp = !c;
        let occ = &self.occupied();
        (     (ATTACK_TABLE.fu.attack(to, opp)      & self.pieces_p(PieceType::FU))
            | (ATTACK_TABLE.ky.attack(to, opp, occ) & self.pieces_p(PieceType::KY))
            | (ATTACK_TABLE.ke.attack(to, opp)      & self.pieces_p(PieceType::KE))
            | (ATTACK_TABLE.gi.attack(to, opp)      & self.pieces_ps(&[PieceType::GI, PieceType::RY, PieceType::OU]))
            | (ATTACK_TABLE.ka.attack(to, occ)      & self.pieces_ps(&[PieceType::KA, PieceType::UM]))
            | (ATTACK_TABLE.hi.attack(to, occ)      & self.pieces_ps(&[PieceType::HI, PieceType::RY]))
            | (ATTACK_TABLE.ki.attack(to, opp)      & self.pieces_ps(&[PieceType::KI, PieceType::TO, PieceType::NY, PieceType::NK, PieceType::NG, PieceType::UM, PieceType::OU]))
        ) & self.pieces_c(c)
    }
    pub fn is_legal_move(&self, m: Move) -> bool {
        if let Some(from) = m.from() {
            let c = self.side_to_move();
            // 玉が相手の攻撃範囲内に動いてしまう指し手は除外
            if self.piece_on(from).piece_type() == Some(PieceType::OU)
                && self.attackers_to(!c, m.to()).is_empty().not()
            {
                return false;
            }
            // 飛び駒から守っている駒が直線上から外れてしまう指し手は除外
            if (self.pinned()[c.index()] & from).is_empty().not() {
                if let Some(sq) = self.king(c) {
                    if (BETWEEN_TABLE[sq.index()][from.index()] & m.to()).is_empty()
                        && (BETWEEN_TABLE[sq.index()][m.to().index()] & from).is_empty()
                    {
                        return false;
                    }
                }
            }
        }
        true
    }
    pub fn is_check_move(&self, m: Move) -> bool {
        let to = m.to();
        let p = m.piece();
        if let Some(from) = m.from() {
            // 直接王手
            let p_to = if m.is_promotion() { p.promoted() } else { p };
            if let Some(pt) = p_to.piece_type() {
                if self.checkable(pt, to) {
                    return true;
                }
            }
            // 開き王手
            let c = self.side_to_move();
            if (self.pinned()[(!c).index()] & from).is_empty().not() {
                if let Some(sq) = self.king(!c) {
                    return (BETWEEN_TABLE[sq.index()][from.index()] & to).is_empty()
                        && (BETWEEN_TABLE[sq.index()][to.index()] & from).is_empty();
                }
            }
        } else {
            let pt = p.piece_type().expect("empty piece for drop move");
            return self.checkable(pt, to);
        }
        false
    }
}

impl Default for Position {
    fn default() -> Self {
        #[rustfmt::skip]
        let board = [
            Piece::WKY, Piece::EMP, Piece::WFU, Piece::EMP, Piece::EMP, Piece::EMP, Piece::BFU, Piece::EMP, Piece::BKY,
            Piece::WKE, Piece::WKA, Piece::WFU, Piece::EMP, Piece::EMP, Piece::EMP, Piece::BFU, Piece::BHI, Piece::BKE,
            Piece::WGI, Piece::EMP, Piece::WFU, Piece::EMP, Piece::EMP, Piece::EMP, Piece::BFU, Piece::EMP, Piece::BGI,
            Piece::WKI, Piece::EMP, Piece::WFU, Piece::EMP, Piece::EMP, Piece::EMP, Piece::BFU, Piece::EMP, Piece::BKI,
            Piece::WOU, Piece::EMP, Piece::WFU, Piece::EMP, Piece::EMP, Piece::EMP, Piece::BFU, Piece::EMP, Piece::BOU,
            Piece::WKI, Piece::EMP, Piece::WFU, Piece::EMP, Piece::EMP, Piece::EMP, Piece::BFU, Piece::EMP, Piece::BKI,
            Piece::WGI, Piece::EMP, Piece::WFU, Piece::EMP, Piece::EMP, Piece::EMP, Piece::BFU, Piece::EMP, Piece::BGI,
            Piece::WKE, Piece::WHI, Piece::WFU, Piece::EMP, Piece::EMP, Piece::EMP, Piece::BFU, Piece::BKA, Piece::BKE,
            Piece::WKY, Piece::EMP, Piece::WFU, Piece::EMP, Piece::EMP, Piece::EMP, Piece::BFU, Piece::EMP, Piece::BKY,
        ];
        Self::new(board, [[0; 7]; 2], Color::Black)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &rank in Rank::ALL.iter() {
            write!(f, "P{}", rank.0 + 1)?;
            for &file in File::ALL.iter().rev() {
                write!(f, "{}", self.piece_on(Square::new(file, rank)))?;
            }
            writeln!(f)?;
        }
        for c in Color::ALL {
            if !self.hands[c.index()].is_empty() {
                write!(
                    f,
                    "P{}",
                    match c {
                        Color::Black => "+",
                        Color::White => "-",
                    }
                )?;
                for &pt in PieceType::ALL_HAND.iter().rev() {
                    for _ in 0..self.hands[c.index()].num(pt) {
                        write!(f, "00{}", pt)?;
                    }
                }
                writeln!(f)?;
            }
        }
        writeln!(
            f,
            "{}",
            match self.side_to_move() {
                Color::Black => "+",
                Color::White => "-",
            }
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let pos = Position::default();
        for sq in Square::ALL {
            #[rustfmt::skip]
            let expected = match sq {
                Square::SQ17 | Square::SQ27 | Square::SQ37 | Square::SQ47 | Square::SQ57 | Square::SQ67 | Square::SQ77 | Square::SQ87 | Square::SQ97 => Piece::BFU,
                Square::SQ19 | Square::SQ99 => Piece::BKY,
                Square::SQ29 | Square::SQ89 => Piece::BKE,
                Square::SQ39 | Square::SQ79 => Piece::BGI,
                Square::SQ49 | Square::SQ69 => Piece::BKI,
                Square::SQ59 => Piece::BOU,
                Square::SQ88 => Piece::BKA,
                Square::SQ28 => Piece::BHI,
                Square::SQ13 | Square::SQ23 | Square::SQ33 | Square::SQ43 | Square::SQ53 | Square::SQ63 | Square::SQ73 | Square::SQ83 | Square::SQ93 => Piece::WFU,
                Square::SQ11 | Square::SQ91 => Piece::WKY,
                Square::SQ21 | Square::SQ81 => Piece::WKE,
                Square::SQ31 | Square::SQ71 => Piece::WGI,
                Square::SQ41 | Square::SQ61 => Piece::WKI,
                Square::SQ51 => Piece::WOU,
                Square::SQ22 => Piece::WKA,
                Square::SQ82 => Piece::WHI,
                _ => Piece::EMP,
            };
            assert_eq!(expected, pos.piece_on(sq), "square: {:?}", sq);
        }
        for c in Color::ALL {
            for pt in PieceType::ALL_HAND {
                assert_eq!(0, pos.hand(c).num(pt));
            }
        }
        assert_eq!(Color::Black, pos.side_to_move());
        assert!(!pos.in_check());
    }

    #[test]
    fn test_do_undo_move() {
        let mut pos = Position::default();
        let moves = [
            Move::new_normal(Square::SQ77, Square::SQ76, false, Piece::BFU),
            Move::new_normal(Square::SQ33, Square::SQ34, false, Piece::WFU),
            Move::new_normal(Square::SQ88, Square::SQ22, true, Piece::BUM),
            Move::new_normal(Square::SQ31, Square::SQ22, false, Piece::WGI),
            Move::new_drop(Square::SQ33, Piece::BKA),
        ];
        // do moves
        for &m in moves.iter() {
            pos.do_move(m);
        }
        // check moved pieces, position states
        for (sq, expected) in [
            (Square::SQ22, Piece::WGI),
            (Square::SQ31, Piece::EMP),
            (Square::SQ33, Piece::BKA),
            (Square::SQ76, Piece::BFU),
            (Square::SQ77, Piece::EMP),
        ] {
            assert_eq!(expected, pos.piece_on(sq), "square: {:?}", sq);
        }
        assert!(pos.hand(Color::Black).is_empty());
        assert!(!pos.hand(Color::White).is_empty());
        assert_eq!(Color::White, pos.side_to_move());
        assert!(pos.in_check());
        // revert to default position
        for &m in moves.iter().rev() {
            pos.undo_move(m);
        }
        let default = Position::default();
        assert!(Square::ALL
            .iter()
            .all(|&sq| pos.piece_on(sq) == default.piece_on(sq)));
        assert_eq!(Color::Black, pos.side_to_move());
        assert!(!pos.in_check());
    }

    #[test]
    fn test_perft() {
        fn perft(pos: &mut Position, depth: usize) -> u64 {
            if depth == 0 {
                return 1;
            }
            let mut count = 0;
            for m in pos.legal_moves() {
                pos.do_move(m);
                count += perft(pos, depth - 1);
                pos.undo_move(m);
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
        {
            // R8/2K1S1SSk/4B4/9/9/9/9/9/1L1L1L3 b RBGSNLP3g3n17p 1
            #[rustfmt::skip]
            let mut pos = Position::new([
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
            assert_eq!(593, perft(&mut pos, 1));
            assert_eq!(105677, perft(&mut pos, 2));
        }
    }

    #[allow(clippy::bool_assert_comparison)]
    #[test]
    fn test_is_check_move() {
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
        #[rustfmt::skip]
        let pos = Position::new([
            Piece::WOU, Piece::EMP, Piece::BKI, Piece::EMP, Piece::BKY, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP,
            Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP,
            Piece::WFU, Piece::EMP, Piece::BFU, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP,
            Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP,
            Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP,
            Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP,
            Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP,
            Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP,
            Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP,
        ], [
            [ 0, 1, 0, 0, 0, 1, 1],
            [16, 2, 4, 4, 3, 1, 1],
        ], Color::Black);
        let test_cases = [
            (Move::new_drop(Square::SQ12, Piece::BKY), true),
            (Move::new_drop(Square::SQ14, Piece::BKY), false),
            (Move::new_drop(Square::SQ22, Piece::BKA), true),
            (Move::new_drop(Square::SQ55, Piece::BKA), false),
            (Move::new_drop(Square::SQ21, Piece::BHI), true),
            (Move::new_drop(Square::SQ51, Piece::BHI), false),
            (
                Move::new_normal(Square::SQ13, Square::SQ12, false, Piece::BKI),
                true,
            ),
            (
                Move::new_normal(Square::SQ13, Square::SQ22, false, Piece::BKI),
                true,
            ),
            (
                Move::new_normal(Square::SQ13, Square::SQ23, false, Piece::BKI),
                true,
            ),
            (
                Move::new_normal(Square::SQ13, Square::SQ14, false, Piece::BKI),
                false,
            ),
        ];
        for (m, expected) in test_cases {
            assert_eq!(expected, pos.is_check_move(m), "{m}");
        }
    }
}
