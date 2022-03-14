use crate::bitboard::Bitboard;
use crate::board_piece::*;
use crate::hand::{Hand, Hands};
use crate::movegen::MoveList;
use crate::piece::PieceType;
use crate::shogi_move::MoveType;
use crate::square::{File, Rank};
use crate::tables::{ATTACK_TABLE, BETWEEN_TABLE};
use crate::zobrist::{Key, ZOBRIST_TABLE};
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
    fn new(checkers: Bitboard, pos: &Position) -> Self {
        let occ = &pos.occupied();
        let opp = !pos.side_to_move();
        let mut pinned = [Bitboard::ZERO, Bitboard::ZERO];
        for c in Color::ALL {
            if let Some(sq) = pos.king(c) {
                #[rustfmt::skip]
                let snipers = (
                      (ATTACK_TABLE.pseudo_attack(PieceType::KY, sq, c) & pos.pieces_p(PieceType::KY))
                    | (ATTACK_TABLE.pseudo_attack(PieceType::KA, sq, c) & (pos.pieces_p(PieceType::KA) | pos.pieces_p(PieceType::UM)))
                    | (ATTACK_TABLE.pseudo_attack(PieceType::HI, sq, c) & (pos.pieces_p(PieceType::HI) | pos.pieces_p(PieceType::RY)))
                ) & pos.pieces_c(!c);
                for sniper in snipers {
                    let blockers = BETWEEN_TABLE[sq.index()][sniper.index()] & *occ;
                    if blockers.count_ones() == 1 {
                        pinned[c.index()] |= blockers;
                    }
                }
            }
        }
        if let Some(sq) = pos.king(opp) {
            let ka = ATTACK_TABLE.ka.attack(sq, occ);
            let hi = ATTACK_TABLE.hi.attack(sq, occ);
            let ki = ATTACK_TABLE.ki.attack(sq, opp);
            let ou = ATTACK_TABLE.ou.attack(sq, opp);
            Self {
                checkers,
                checkables: [
                    ATTACK_TABLE.fu.attack(sq, opp),
                    ATTACK_TABLE.ky.attack(sq, opp, occ),
                    ATTACK_TABLE.ke.attack(sq, opp),
                    ATTACK_TABLE.gi.attack(sq, opp),
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
    fn calculate_checkers(pos: &Position) -> Bitboard {
        let opp = !pos.side_to_move();
        let occ = &pos.occupied();
        if let Some(sq) = pos.king(opp) {
            (     (ATTACK_TABLE.fu.attack(sq, opp)      & pos.pieces_p(PieceType::FU))
                | (ATTACK_TABLE.ky.attack(sq, opp, occ) & pos.pieces_p(PieceType::KY))
                | (ATTACK_TABLE.ke.attack(sq, opp)      & pos.pieces_p(PieceType::KE))
                | (ATTACK_TABLE.gi.attack(sq, opp)      & (pos.pieces_p(PieceType::GI) | pos.pieces_p(PieceType::RY)))
                | (ATTACK_TABLE.ka.attack(sq, occ)      & (pos.pieces_p(PieceType::KA) | pos.pieces_p(PieceType::UM)))
                | (ATTACK_TABLE.hi.attack(sq, occ)      & (pos.pieces_p(PieceType::HI) | pos.pieces_p(PieceType::RY)))
                | (ATTACK_TABLE.ki.attack(sq, opp)      & (pos.pieces_p(PieceType::KI) | pos.pieces_p(PieceType::TO) | pos.pieces_p(PieceType::NY) | pos.pieces_p(PieceType::NK) | pos.pieces_p(PieceType::NG) | pos.pieces_p(PieceType::UM)))
            ) & pos.pieces_c(pos.side_to_move())
        } else {
            Bitboard::ZERO
        }
    }
}

#[derive(Debug)]
struct State {
    keys: (Key, Key),
    captured: Option<Piece>,
    attack_info: AttackInfo,
}

impl State {
    fn new(keys: (Key, Key), captured: Option<Piece>, attack_info: AttackInfo) -> Self {
        Self {
            keys,
            captured,
            attack_info,
        }
    }
}

/// Represents a state of the game.
#[derive(Debug)]
pub struct Position {
    board: [Option<Piece>; Square::NUM],
    hands: Hands,
    color: Color,
    ply: u32,
    color_bbs: [Bitboard; Color::NUM],
    piece_type_bbs: [Bitboard; PieceType::NUM],
    occupied_bb: Bitboard,
    states: Vec<State>,
}

impl Position {
    pub fn new(
        board: [Option<Piece>; Square::NUM],
        hand_nums: [[u8; PieceType::NUM_HAND]; Color::NUM],
        side_to_move: Color,
        ply: u32,
    ) -> Position {
        let mut keys = (Key::ZERO, Key::ZERO);
        // board
        let mut color_bbs = [Bitboard::ZERO; Color::NUM];
        let mut piece_type_bbs = [Bitboard::ZERO; PieceType::NUM];
        let mut occupied_bb = Bitboard::ZERO;
        for sq in Square::ALL {
            if let Some(p) = board[sq.index()] {
                color_bbs[p.color().index()] |= sq;
                piece_type_bbs[p.piece_type().index()] |= sq;
                occupied_bb |= sq;
                keys.0 ^= ZOBRIST_TABLE.board(sq, p);
            }
        }
        // hands
        let mut hands = [Hand::new(); Color::NUM];
        for c in Color::ALL {
            for (&num, &pt) in hand_nums[c.index()].iter().zip(PieceType::ALL_HAND.iter()) {
                for i in 0..num {
                    hands[c.index()].increment(pt);
                    keys.1 ^= ZOBRIST_TABLE.hand(c, pt, i + 1)
                }
            }
        }
        // new position with the opposite side_to_move for calculating checkers
        let mut pos = Self {
            board,
            hands: Hands::new(hands),
            color: !side_to_move,
            ply,
            color_bbs,
            piece_type_bbs,
            occupied_bb,
            states: Vec::new(),
        };
        // create initial state
        let checkers = AttackInfo::calculate_checkers(&pos);
        pos.color = side_to_move;
        pos.states
            .push(State::new(keys, None, AttackInfo::new(checkers, &pos)));
        pos
    }
    pub fn hand(&self, c: Color) -> Hand {
        self.hands.hand(c)
    }
    pub fn side_to_move(&self) -> Color {
        self.color
    }
    pub fn ply(&self) -> u32 {
        self.ply
    }
    pub fn piece_on(&self, sq: Square) -> Option<Piece> {
        self.board[sq.index()]
    }
    pub fn pieces_cp(&self, c: Color, pt: PieceType) -> Bitboard {
        self.pieces_c(c) & self.pieces_p(pt)
    }
    pub fn pieces_c(&self, c: Color) -> Bitboard {
        self.color_bbs[c.index()]
    }
    pub fn pieces_p(&self, pt: PieceType) -> Bitboard {
        self.piece_type_bbs[pt.index()]
    }
    pub fn occupied(&self) -> Bitboard {
        self.occupied_bb
    }
    pub fn key(&self) -> u64 {
        (self.state().keys.0 ^ self.state().keys.1).value()
    }
    pub fn keys(&self) -> (u64, u64) {
        (self.state().keys.0.value(), self.state().keys.1.value())
    }
    pub fn captured(&self) -> Option<Piece> {
        self.state().captured
    }
    pub fn checkers(&self) -> Bitboard {
        self.state().attack_info.checkers
    }
    pub fn in_check(&self) -> bool {
        self.checkers().is_empty().not()
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
    pub fn legal_moves(&self) -> MoveList {
        let mut ml = MoveList::default();
        ml.generate_legals(self);
        ml
    }
    pub fn do_move(&mut self, m: Move) {
        let is_check = self.is_check_move(m);
        let captured = self.piece_on(m.to());
        let c = self.side_to_move();
        let mut keys = self.state().keys;
        let checkers = match m.move_type() {
            // 駒移動
            MoveType::Normal {
                from,
                to,
                is_promotion,
                piece,
            } => {
                self.remove_piece(from, piece);
                // 移動先に駒がある場合
                if let Some(p) = captured {
                    let pt = p.piece_type();
                    self.xor_bbs(!c, pt, to);
                    self.hands.increment(c, pt);
                    let num = self.hand(c).num(pt);
                    keys.0 ^= ZOBRIST_TABLE.board(to, p);
                    keys.1 ^= ZOBRIST_TABLE.hand(c, pt, num);
                }
                let p = if is_promotion {
                    piece.promoted()
                } else {
                    piece
                };
                self.put_piece(to, p);
                keys.0 ^= ZOBRIST_TABLE.board(from, piece);
                keys.0 ^= ZOBRIST_TABLE.board(to, p);
                if is_check {
                    AttackInfo::calculate_checkers(self)
                } else {
                    Bitboard::ZERO
                }
            }
            // 駒打ち
            MoveType::Drop { to, piece } => {
                let pt = piece.piece_type();
                let num = self.hand(c).num(pt);
                self.put_piece(to, piece);
                keys.1 ^= ZOBRIST_TABLE.hand(c, pt, num);
                self.hands.decrement(c, pt);
                keys.0 ^= ZOBRIST_TABLE.board(to, piece);
                if is_check {
                    Bitboard::from_square(to)
                } else {
                    Bitboard::ZERO
                }
            }
        };
        self.color = !c;
        keys.0 ^= Key::COLOR;
        self.ply += 1;
        self.states
            .push(State::new(keys, captured, AttackInfo::new(checkers, self)));
    }
    pub fn undo_move(&mut self, m: Move) {
        let c = self.side_to_move();
        match m.move_type() {
            MoveType::Normal {
                from,
                to,
                is_promotion,
                piece,
            } => {
                let p_to = if is_promotion {
                    piece.promoted()
                } else {
                    piece
                };
                self.remove_piece(to, p_to);
                if let Some(p_cap) = self.captured() {
                    self.put_piece(to, p_cap);
                    self.hands.decrement(!c, p_cap.piece_type());
                }
                self.put_piece(from, piece);
            }
            // 駒打ち
            MoveType::Drop { to, piece } => {
                self.remove_piece(to, piece);
                self.hands.increment(!c, piece.piece_type());
            }
        }
        self.color = !self.color;
        self.ply -= 1;
        self.states.pop();
    }
    fn state(&self) -> &State {
        self.states.last().expect("empty states")
    }
    fn put_piece(&mut self, sq: Square, p: Piece) {
        self.xor_bbs(p.color(), p.piece_type(), sq);
        self.board[sq.index()] = Some(p);
    }
    fn remove_piece(&mut self, sq: Square, p: Piece) {
        self.xor_bbs(p.color(), p.piece_type(), sq);
        self.board[sq.index()] = None;
    }
    fn xor_bbs(&mut self, c: Color, pt: PieceType, sq: Square) {
        self.color_bbs[c.index()] ^= sq;
        self.piece_type_bbs[pt.index()] ^= sq;
        self.occupied_bb ^= sq;
    }
    #[rustfmt::skip]
    pub fn attackers_to(&self, c: Color, to: Square) -> Bitboard {
        let opp = !c;
        let occ = &self.occupied();
        (     (ATTACK_TABLE.fu.attack(to, opp)      & self.pieces_p(PieceType::FU))
            | (ATTACK_TABLE.ky.attack(to, opp, occ) & self.pieces_p(PieceType::KY))
            | (ATTACK_TABLE.ke.attack(to, opp)      & self.pieces_p(PieceType::KE))
            | (ATTACK_TABLE.gi.attack(to, opp)      & (self.pieces_p(PieceType::GI) | self.pieces_p(PieceType::RY) | self.pieces_p(PieceType::OU)))
            | (ATTACK_TABLE.ka.attack(to, occ)      & (self.pieces_p(PieceType::KA) | self.pieces_p(PieceType::UM)))
            | (ATTACK_TABLE.hi.attack(to, occ)      & (self.pieces_p(PieceType::HI) | self.pieces_p(PieceType::RY)))
            | (ATTACK_TABLE.ki.attack(to, opp)      & (self.pieces_p(PieceType::KI) | self.pieces_p(PieceType::TO) | self.pieces_p(PieceType::NY) | self.pieces_p(PieceType::NK) | self.pieces_p(PieceType::NG) | self.pieces_p(PieceType::UM) | self.pieces_p( PieceType::OU)))
        ) & self.pieces_c(c)
    }
    pub fn is_legal_move(&self, m: Move) -> bool {
        if let Some(from) = m.from() {
            let c = self.side_to_move();
            // 玉が相手の攻撃範囲内に動いてしまう指し手は除外
            if self.piece_on(from) == Some(Piece::from_cp(c, PieceType::OU))
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
        match m.move_type() {
            MoveType::Normal {
                from,
                to,
                is_promotion,
                piece,
            } => {
                // 直接王手
                let p = if is_promotion {
                    piece.promoted()
                } else {
                    piece
                };
                if self.checkable(p.piece_type(), to) {
                    return true;
                }
                // 開き王手
                let c = self.side_to_move();
                if (self.pinned()[(!c).index()] & from).is_empty().not() {
                    if let Some(sq) = self.king(!c) {
                        return (BETWEEN_TABLE[sq.index()][from.index()] & to).is_empty()
                            && (BETWEEN_TABLE[sq.index()][to.index()] & from).is_empty();
                    }
                }
                false
            }
            MoveType::Drop { to, piece } => self.checkable(piece.piece_type(), to),
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        #[rustfmt::skip]
        let board = [
            WKY, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BKY,
            WKE, WKA, WFU, EMP, EMP, EMP, BFU, BHI, BKE,
            WGI, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BGI,
            WKI, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BKI,
            WOU, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BOU,
            WKI, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BKI,
            WGI, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BGI,
            WKE, WHI, WFU, EMP, EMP, EMP, BFU, BKA, BKE,
            WKY, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BKY,
        ];
        Self::new(board, [[0; 7]; 2], Color::Black, 1)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &rank in Rank::ALL.iter() {
            write!(f, "P{rank}")?;
            for &file in File::ALL.iter().rev() {
                if let Some(p) = self.piece_on(Square::new(file, rank)) {
                    write!(f, "{p}")?;
                } else {
                    write!(f, " * ")?;
                }
            }
            writeln!(f)?;
        }
        write!(f, "{}", self.hands)?;
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
    fn default() {
        let pos = Position::default();
        for sq in Square::ALL {
            #[rustfmt::skip]
            let expected = match sq {
                Square::SQ17 | Square::SQ27 | Square::SQ37 | Square::SQ47 | Square::SQ57 | Square::SQ67 | Square::SQ77 | Square::SQ87 | Square::SQ97 => Some(Piece::BFU),
                Square::SQ19 | Square::SQ99 => Some(Piece::BKY),
                Square::SQ29 | Square::SQ89 => Some(Piece::BKE),
                Square::SQ39 | Square::SQ79 => Some(Piece::BGI),
                Square::SQ49 | Square::SQ69 => Some(Piece::BKI),
                Square::SQ59 => Some(Piece::BOU),
                Square::SQ88 => Some(Piece::BKA),
                Square::SQ28 => Some(Piece::BHI),
                Square::SQ13 | Square::SQ23 | Square::SQ33 | Square::SQ43 | Square::SQ53 | Square::SQ63 | Square::SQ73 | Square::SQ83 | Square::SQ93 => Some(Piece::WFU),
                Square::SQ11 | Square::SQ91 => Some(Piece::WKY),
                Square::SQ21 | Square::SQ81 => Some(Piece::WKE),
                Square::SQ31 | Square::SQ71 => Some(Piece::WGI),
                Square::SQ41 | Square::SQ61 => Some(Piece::WKI),
                Square::SQ51 => Some(Piece::WOU),
                Square::SQ22 => Some(Piece::WKA),
                Square::SQ82 => Some(Piece::WHI),
                _ => None,
            };
            assert_eq!(expected, pos.piece_on(sq), "square: {:?}", sq);
        }
        for c in Color::ALL {
            for pt in PieceType::ALL_HAND {
                assert_eq!(0, pos.hand(c).num(pt));
            }
        }
        assert_eq!(Color::Black, pos.side_to_move());
        assert_eq!(1, pos.ply());
        assert!(!pos.in_check());
    }

    #[test]
    fn do_undo_move() {
        let mut pos = Position::default();
        let moves = [
            Move::new_normal(Square::SQ77, Square::SQ76, false, Piece::BFU),
            Move::new_normal(Square::SQ33, Square::SQ34, false, Piece::WFU),
            Move::new_normal(Square::SQ88, Square::SQ22, true, Piece::BKA),
            Move::new_normal(Square::SQ31, Square::SQ22, false, Piece::WGI),
            Move::new_drop(Square::SQ33, Piece::BKA),
        ];
        // do moves
        for &m in moves.iter() {
            pos.do_move(m);
        }
        // check moved pieces, position states
        for (sq, expected) in [
            (Square::SQ22, Some(Piece::WGI)),
            (Square::SQ31, None),
            (Square::SQ33, Some(Piece::BKA)),
            (Square::SQ76, Some(Piece::BFU)),
            (Square::SQ77, None),
        ] {
            assert_eq!(expected, pos.piece_on(sq), "square: {:?}", sq);
        }
        assert!(pos.hand(Color::Black).is_empty());
        assert!(!pos.hand(Color::White).is_empty());
        assert_eq!(Color::White, pos.side_to_move());
        assert_eq!(6, pos.ply());
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
        assert_eq!(1, pos.ply());
        assert!(!pos.in_check());
    }

    #[test]
    fn perft() {
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
                EMP, WOU, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                EMP, BGI, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                EMP, BGI, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, BKY,
                EMP, BGI, BKA, EMP, EMP, EMP, EMP, EMP, EMP,
                EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, BKY,
                EMP, BOU, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, BKY,
                BHI, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP
            ], [
                [ 1, 1, 1, 1, 1, 1, 1],
                [17, 0, 3, 0, 3, 0, 0],
            ], Color::Black, 1);
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
        #[rustfmt::skip]
        let pos = Position::new([
            WOU, EMP, BKI, EMP, BKY, EMP, EMP, EMP, EMP,
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
            WFU, EMP, BFU, EMP, EMP, EMP, EMP, EMP, EMP,
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
        ], [
            [ 0, 1, 0, 0, 0, 1, 1],
            [16, 2, 4, 4, 3, 1, 1],
        ], Color::Black, 1);
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
