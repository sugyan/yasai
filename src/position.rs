use crate::board_piece::*;
use crate::movegen::MoveList;
use crate::shogi_move::MoveType;
use crate::tables::{ATTACK_TABLE, BETWEEN_TABLE};
use crate::zobrist::{Key, ZOBRIST_TABLE};
use crate::Move;
use shogi_core::{Bitboard, Color, Hand, Piece, PieceKind, Square};

#[derive(Debug, Clone)]
struct AttackInfo {
    checkers: Bitboard,                     // 王手をかけている駒の位置
    checkables: [Bitboard; PieceKind::NUM], // 各駒種が王手になり得る位置
    pinned: [Bitboard; 2],                  // 飛び駒から玉を守っている駒の位置
}

impl AttackInfo {
    fn new(checkers: Bitboard, pos: &Position) -> Self {
        let occ = &pos.occupied();
        let opp = pos.side_to_move().flip();
        let mut pinned = [Bitboard::empty(), Bitboard::empty()];
        for c in Color::all() {
            if let Some(sq) = pos.king(c) {
                #[rustfmt::skip]
                let snipers = (
                      (ATTACK_TABLE.pseudo_attack(PieceKind::Lance, sq, c) & pos.pieces_p(PieceKind::Lance))
                    | (ATTACK_TABLE.pseudo_attack(PieceKind::Bishop, sq, c) & (pos.pieces_p(PieceKind::Bishop) | pos.pieces_p(PieceKind::ProBishop)))
                    | (ATTACK_TABLE.pseudo_attack(PieceKind::Rook, sq, c) & (pos.pieces_p(PieceKind::Rook) | pos.pieces_p(PieceKind::ProRook)))
                ) & pos.pieces_c(c.flip());
                for sniper in snipers {
                    let blockers = BETWEEN_TABLE[sq.array_index()][sniper.array_index()] & *occ;
                    if blockers.count() == 1 {
                        pinned[c.array_index()] |= blockers;
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
                    ki,
                    ka,
                    hi,
                    Bitboard::empty(),
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
                checkables: [Bitboard::empty(); PieceKind::NUM],
                pinned,
            }
        }
    }
    #[rustfmt::skip]
    fn calculate_checkers(pos: &Position) -> Bitboard {
        let opp = pos.side_to_move().flip();
        let occ = &pos.occupied();
        if let Some(sq) = pos.king(opp) {
            (     (ATTACK_TABLE.fu.attack(sq, opp)      & pos.pieces_p(PieceKind::Pawn))
                | (ATTACK_TABLE.ky.attack(sq, opp, occ) & pos.pieces_p(PieceKind::Lance))
                | (ATTACK_TABLE.ke.attack(sq, opp)      & pos.pieces_p(PieceKind::Knight))
                | (ATTACK_TABLE.gi.attack(sq, opp)      & (pos.pieces_p(PieceKind::Silver) | pos.pieces_p(PieceKind::ProRook)))
                | (ATTACK_TABLE.ka.attack(sq, occ)      & (pos.pieces_p(PieceKind::Bishop) | pos.pieces_p(PieceKind::ProBishop)))
                | (ATTACK_TABLE.hi.attack(sq, occ)      & (pos.pieces_p(PieceKind::Rook) | pos.pieces_p(PieceKind::ProRook)))
                | (ATTACK_TABLE.ki.attack(sq, opp)      & (pos.pieces_p(PieceKind::Gold) | pos.pieces_p(PieceKind::ProPawn) | pos.pieces_p(PieceKind::ProLance) | pos.pieces_p(PieceKind::ProKnight) | pos.pieces_p(PieceKind::ProSilver) | pos.pieces_p(PieceKind::ProBishop)))
            ) & pos.pieces_c(pos.side_to_move())
        } else {
            Bitboard::empty()
        }
    }
}

#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct Position {
    board: [Option<Piece>; Square::NUM],
    hands: [Hand; 2],
    color: Color,
    ply: u32,
    color_bbs: [Bitboard; 2],
    piece_type_bbs: [Bitboard; PieceKind::NUM],
    occupied_bb: Bitboard,
    states: Vec<State>,
}

impl Position {
    pub fn new(
        board: [Option<Piece>; Square::NUM],
        hand_nums: [[u8; 8]; 2],
        side_to_move: Color,
        ply: u32,
    ) -> Position {
        let mut keys = (Key::ZERO, Key::ZERO);
        // board
        let mut color_bbs = [Bitboard::empty(); 2];
        let mut piece_type_bbs = [Bitboard::empty(); PieceKind::NUM];
        let mut occupied_bb = Bitboard::empty();
        for sq in Square::all() {
            if let Some(p) = board[sq.array_index()] {
                color_bbs[p.color().array_index()] |= sq;
                piece_type_bbs[p.piece_kind().array_index()] |= sq;
                occupied_bb |= sq;
                keys.0 ^= ZOBRIST_TABLE.board(sq, p);
            }
        }
        // hands
        let mut hands = [Hand::new(); 2];
        for c in Color::all() {
            for (i, &num) in hand_nums[c.array_index()].iter().enumerate() {
                if let Some(pk) = PieceKind::from_u8(i as u8 + 1) {
                    for j in 0..num {
                        if let Some(h) = hands[c.array_index()].added(pk) {
                            hands[c.array_index()] = h;
                        }
                        keys.1 ^= ZOBRIST_TABLE.hand(c, pk, j);
                    }
                }
            }
        }
        // new position with the opposite side_to_move for calculating checkers
        let mut pos = Self {
            board,
            hands,
            color: side_to_move.flip(),
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
        self.hands[c.array_index()]
    }
    pub fn side_to_move(&self) -> Color {
        self.color
    }
    pub fn ply(&self) -> u32 {
        self.ply
    }
    pub fn piece_on(&self, sq: Square) -> Option<Piece> {
        self.board[sq.array_index()]
    }
    pub fn pieces_cp(&self, c: Color, pk: PieceKind) -> Bitboard {
        self.pieces_c(c) & self.pieces_p(pk)
    }
    pub fn pieces_c(&self, c: Color) -> Bitboard {
        self.color_bbs[c.array_index()]
    }
    pub fn pieces_p(&self, pk: PieceKind) -> Bitboard {
        self.piece_type_bbs[pk.array_index()]
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
        !self.checkers().is_empty()
    }
    fn checkable(&self, pk: PieceKind, sq: Square) -> bool {
        !(self.state().attack_info.checkables[pk.array_index()] & Bitboard::single(sq)).is_empty()
    }
    pub fn pinned(&self) -> [Bitboard; 2] {
        self.state().attack_info.pinned
    }
    pub fn king(&self, c: Color) -> Option<Square> {
        self.pieces_cp(c, PieceKind::King).pop()
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
                    let pk = p.piece_kind();
                    self.xor_bbs(c.flip(), pk, to);
                    let pk_unpromoted = if let Some(pk) = pk.unpromote() {
                        pk
                    } else {
                        pk
                    };
                    let h = self.hands[c.array_index()]
                        .added(pk_unpromoted)
                        .expect("invalid piece kind");
                    self.hands[c.array_index()] = h;
                    let num = h.count(pk_unpromoted).expect("invalid piece kind");
                    keys.0 ^= ZOBRIST_TABLE.board(to, p);
                    keys.1 ^= ZOBRIST_TABLE.hand(c, pk_unpromoted, num - 1);
                }
                let p = if is_promotion {
                    if let Some(p) = piece.promote() {
                        p
                    } else {
                        piece
                    }
                } else {
                    piece
                };
                self.put_piece(to, p);
                keys.0 ^= ZOBRIST_TABLE.board(from, piece);
                keys.0 ^= ZOBRIST_TABLE.board(to, p);
                if is_check {
                    AttackInfo::calculate_checkers(self)
                } else {
                    Bitboard::empty()
                }
            }
            // 駒打ち
            MoveType::Drop { to, piece } => {
                let pk = piece.piece_kind();
                let num = self.hands[c.array_index()]
                    .count(pk)
                    .expect("invalid piece kind");
                self.put_piece(to, piece);
                keys.1 ^= ZOBRIST_TABLE.hand(c, pk, num);
                self.hands[c.array_index()] = self.hands[c.array_index()]
                    .removed(pk)
                    .expect("invalid piece kind");
                keys.0 ^= ZOBRIST_TABLE.board(to, piece);
                if is_check {
                    Bitboard::single(to)
                } else {
                    Bitboard::empty()
                }
            }
        };
        self.color = c.flip();
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
                    if let Some(p) = piece.promote() {
                        p
                    } else {
                        piece
                    }
                } else {
                    piece
                };
                self.remove_piece(to, p_to);
                if let Some(p_cap) = self.captured() {
                    let pk = p_cap.piece_kind();
                    self.put_piece(to, p_cap);
                    let pk_unpromoted = if let Some(pk) = pk.unpromote() {
                        pk
                    } else {
                        pk
                    };
                    self.hands[c.flip().array_index()] = self.hands[c.flip().array_index()]
                        .removed(pk_unpromoted)
                        .expect("invalid piece kind");
                }
                self.put_piece(from, piece);
            }
            // 駒打ち
            MoveType::Drop { to, piece } => {
                self.remove_piece(to, piece);
                self.hands[c.flip().array_index()] = self.hands[c.flip().array_index()]
                    .added(piece.piece_kind())
                    .expect("invalid piece kind");
            }
        }
        self.color = self.color.flip();
        self.ply -= 1;
        self.states.pop();
    }
    fn state(&self) -> &State {
        self.states.last().expect("empty states")
    }
    fn put_piece(&mut self, sq: Square, p: Piece) {
        self.xor_bbs(p.color(), p.piece_kind(), sq);
        self.board[sq.array_index()] = Some(p);
    }
    fn remove_piece(&mut self, sq: Square, p: Piece) {
        self.xor_bbs(p.color(), p.piece_kind(), sq);
        self.board[sq.array_index()] = None;
    }
    fn xor_bbs(&mut self, c: Color, pk: PieceKind, sq: Square) {
        self.color_bbs[c.array_index()] ^= sq;
        self.piece_type_bbs[pk.array_index()] ^= sq;
        self.occupied_bb ^= sq;
    }
    #[rustfmt::skip]
    pub fn attackers_to(&self, c: Color, to: Square, occ: &Bitboard) -> Bitboard {
        let opp = c.flip();
        (     (ATTACK_TABLE.fu.attack(to, opp)      & self.pieces_p(PieceKind::Pawn))
            | (ATTACK_TABLE.ky.attack(to, opp, occ) & self.pieces_p(PieceKind::Lance))
            | (ATTACK_TABLE.ke.attack(to, opp)      & self.pieces_p(PieceKind::Knight))
            | (ATTACK_TABLE.gi.attack(to, opp)      & (self.pieces_p(PieceKind::Silver) | self.pieces_p(PieceKind::ProRook) | self.pieces_p(PieceKind::King)))
            | (ATTACK_TABLE.ka.attack(to, occ)      & (self.pieces_p(PieceKind::Bishop) | self.pieces_p(PieceKind::ProBishop)))
            | (ATTACK_TABLE.hi.attack(to, occ)      & (self.pieces_p(PieceKind::Rook) | self.pieces_p(PieceKind::ProRook)))
            | (ATTACK_TABLE.ki.attack(to, opp)      & (self.pieces_p(PieceKind::Gold) | self.pieces_p(PieceKind::ProPawn) | self.pieces_p(PieceKind::ProLance) | self.pieces_p(PieceKind::ProKnight) | self.pieces_p(PieceKind::ProSilver) | self.pieces_p(PieceKind::ProBishop) | self.pieces_p( PieceKind::King)))
        ) & self.pieces_c(c)
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
                let c = self.side_to_move();
                if !(self.pinned()[c.flip().array_index()] & Bitboard::single(from)).is_empty() {
                    if let Some(sq) = self.king(c.flip()) {
                        return (BETWEEN_TABLE[sq.array_index()][from.array_index()]
                            & Bitboard::single(to))
                        .is_empty()
                            && (BETWEEN_TABLE[sq.array_index()][to.array_index()]
                                & Bitboard::single(from))
                            .is_empty();
                    }
                }
                false
            }
            MoveType::Drop { to, piece } => self.checkable(piece.piece_kind(), to),
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
        Self::new(board, [[0; 8]; 2], Color::Black, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            assert_eq!(expected, pos.piece_on(sq), "square: {:?}", sq);
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
            Move::new_normal(Square::SQ_7G, Square::SQ_7F, false, Piece::B_P),
            Move::new_normal(Square::SQ_3C, Square::SQ_3D, false, Piece::W_P),
            Move::new_normal(Square::SQ_8H, Square::SQ_2B, true, Piece::B_B),
            Move::new_normal(Square::SQ_3A, Square::SQ_2B, false, Piece::W_S),
            Move::new_drop(Square::SQ_3C, Piece::B_B),
        ];
        // do moves
        for &m in moves.iter() {
            pos.do_move(m);
        }
        // check moved pieces, position states
        for (sq, expected) in [
            (Square::SQ_2B, Some(Piece::W_S)),
            (Square::SQ_3A, None),
            (Square::SQ_3C, Some(Piece::B_B)),
            (Square::SQ_7F, Some(Piece::B_P)),
            (Square::SQ_7G, None),
        ] {
            assert_eq!(expected, pos.piece_on(sq), "square: {:?}", sq);
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
            pos.undo_move(m);
        }
        let default = Position::default();
        assert!(Square::all().all(|sq| pos.piece_on(sq) == default.piece_on(sq)));
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
                [ 1, 1, 1, 1, 1, 1, 1, 0],
                [17, 0, 3, 0, 3, 0, 0, 0],
            ], Color::Black, 1);
            assert_eq!(593, perft(&mut pos, 1));
            assert_eq!(105677, perft(&mut pos, 2));
        }
    }

    #[allow(clippy::bool_assert_comparison)]
    #[test]
    fn is_check_move() {
        #[rustfmt::skip]
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
            [ 0, 1, 0, 0, 0, 1, 1, 0],
            [16, 2, 4, 4, 3, 1, 1, 0],
        ], Color::Black, 1);
        let test_cases = [
            (Move::new_drop(Square::SQ_1B, Piece::B_L), true),
            (Move::new_drop(Square::SQ_1D, Piece::B_L), false),
            (Move::new_drop(Square::SQ_2B, Piece::B_B), true),
            (Move::new_drop(Square::SQ_5E, Piece::B_B), false),
            (Move::new_drop(Square::SQ_2A, Piece::B_R), true),
            (Move::new_drop(Square::SQ_5A, Piece::B_R), false),
            (
                Move::new_normal(Square::SQ_1C, Square::SQ_1B, false, Piece::B_G),
                true,
            ),
            (
                Move::new_normal(Square::SQ_1C, Square::SQ_2B, false, Piece::B_G),
                true,
            ),
            (
                Move::new_normal(Square::SQ_1C, Square::SQ_2C, false, Piece::B_G),
                true,
            ),
            (
                Move::new_normal(Square::SQ_1C, Square::SQ_1D, false, Piece::B_G),
                false,
            ),
        ];
        for (m, expected) in test_cases {
            assert_eq!(expected, pos.is_check_move(m));
        }
    }
}
