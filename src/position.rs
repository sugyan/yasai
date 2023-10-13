use crate::bitboard::Bitboard;
use crate::tables::{ATTACK_TABLE, BETWEEN_TABLE};
use crate::zobrist::{Key, ZOBRIST_TABLE};
use shogi_core::{Color, Hand, Move, Piece, PieceKind, Square};

/// Represents a state of the game with history. This provides the ability to do and undo moves.
#[derive(Debug, Clone)]
pub struct Position {
    inner: PartialPosition,
    /// History of the positions
    states: Vec<State>,
}

impl Position {
    pub fn new(partial: shogi_core::PartialPosition) -> Position {
        let inner = PartialPosition::from(partial);
        let mut keys = (Key::ZERO, Key::ZERO);
        for sq in Square::all() {
            if let Some(p) = inner.board[sq.array_index()] {
                keys.0 ^= ZOBRIST_TABLE.board(sq, p);
            }
        }
        for c in Color::all() {
            for pk in Hand::all_hand_pieces() {
                if let Some(num) = inner.hands[c.array_index()].count(pk) {
                    for i in 0..num {
                        keys.1 ^= ZOBRIST_TABLE.hand(c, pk, i);
                    }
                }
            }
        }
        let checkers = AttackInfo::calculate_checkers(&inner);
        let state = State {
            keys,
            captured: None,
            last_moved: None,
            attack_info: AttackInfo::new(checkers, &inner),
        };
        Self {
            inner,
            states: vec![state],
        }
    }
    #[inline(always)]
    pub fn side_to_move(&self) -> Color {
        self.inner.side
    }
    #[inline(always)]
    pub fn ply(&self) -> u16 {
        self.inner.ply
    }
    #[inline(always)]
    pub fn hand(&self, color: Color) -> Hand {
        self.inner.hands[color.array_index()]
    }
    #[inline(always)]
    pub fn piece_at(&self, sq: Square) -> Option<Piece> {
        self.inner.piece_at(sq)
    }
    #[inline(always)]
    pub fn key(&self) -> u64 {
        (self.state().keys.0 ^ self.state().keys.1).value()
    }
    #[inline(always)]
    pub fn keys(&self) -> (u64, u64) {
        (self.state().keys.0.value(), self.state().keys.1.value())
    }
    #[inline(always)]
    pub fn in_check(&self) -> bool {
        !self.checkers().is_empty()
    }
    pub fn is_check_move(&self, m: Move) -> bool {
        match m {
            Move::Normal { from, to, promote } => {
                let piece = self.inner.piece_at(from).unwrap();
                let pk = piece.piece_kind();
                let pk = if promote {
                    pk.promote().unwrap_or(pk)
                } else {
                    pk
                };
                if self.checkable(pk, to) {
                    return true;
                }
                // 開き王手
                let c = self.inner.side.flip();
                if self.pinned(c).contains(from) {
                    let sq = self.king_position(c).unwrap();
                    return !(BETWEEN_TABLE[sq.array_index()][from.array_index()].contains(to)
                        || BETWEEN_TABLE[sq.array_index()][to.array_index()].contains(from));
                }
                false
            }
            Move::Drop { to, piece } => self.checkable(piece.piece_kind(), to),
        }
    }
    pub fn do_move(&mut self, m: Move) {
        let c = self.side_to_move();
        let is_check = self.is_check_move(m);
        let captured = self.inner.piece_at(m.to());
        let last_moved;
        let mut keys = self.state().keys;
        let checkers = match m {
            Move::Normal { from, to, promote } => {
                let piece = self.inner.piece_at(from).unwrap();
                last_moved = Some(piece);
                if let Some(p) = captured {
                    let pk = p.piece_kind();
                    let pk_unpromoted = pk.unpromote().unwrap_or(pk);
                    // Update keys
                    keys.0 ^= ZOBRIST_TABLE.board(to, p);
                    keys.1 ^= ZOBRIST_TABLE.hand(
                        c,
                        pk_unpromoted,
                        self.inner.hand_of_a_player(c).count(pk_unpromoted).unwrap(),
                    );
                    // Update inner state
                    self.inner.xor_piece(to, p);
                    let hand = self.inner.hand_of_a_player_mut(c);
                    *hand = hand.added(pk_unpromoted).unwrap();
                }
                let target_piece = if promote {
                    piece.promote().unwrap()
                } else {
                    piece
                };
                // Update inner state
                self.inner.xor_piece(from, piece);
                self.inner.xor_piece(to, target_piece);
                *self.inner.piece_at_mut(from) = None;
                *self.inner.piece_at_mut(to) = Some(target_piece);
                self.inner.side = c.flip();
                // Update keys
                keys.0 ^= ZOBRIST_TABLE.board(from, piece);
                keys.0 ^= ZOBRIST_TABLE.board(to, target_piece);
                if is_check {
                    AttackInfo::calculate_checkers(&self.inner)
                } else {
                    Bitboard::empty()
                }
            }
            Move::Drop { to, piece } => {
                last_moved = Some(piece);
                // Update inner state
                self.inner.xor_piece(to, piece);
                *self.inner.piece_at_mut(to) = Some(piece);
                let hand = self.inner.hand_of_a_player_mut(c);
                *hand = hand.removed(piece.piece_kind()).unwrap();
                self.inner.side = c.flip();
                // Update keys
                keys.1 ^= ZOBRIST_TABLE.hand(
                    c,
                    piece.piece_kind(),
                    self.inner
                        .hand_of_a_player(c)
                        .count(piece.piece_kind())
                        .unwrap(),
                );
                keys.0 ^= ZOBRIST_TABLE.board(to, piece);
                if is_check {
                    Bitboard::single(to)
                } else {
                    Bitboard::empty()
                }
            }
        };
        self.inner.ply += 1;
        keys.0 ^= Key::COLOR;
        self.states.push(State {
            keys,
            captured,
            last_moved,
            attack_info: AttackInfo::new(checkers, &self.inner),
        });
    }
    pub fn undo_move(&mut self, m: Move) {
        let c = self.side_to_move().flip();
        match m {
            Move::Normal {
                from,
                to,
                promote: _,
            } => {
                let last_moved = self.last_moved().unwrap();
                let captured = self.captured();
                if let Some(p_cap) = captured {
                    let pk = p_cap.piece_kind();
                    let pk_unpromoted = pk.unpromote().unwrap_or(pk);
                    self.inner.xor_piece(to, p_cap);
                    let hand = self.inner.hand_of_a_player_mut(c);
                    *hand = hand.removed(pk_unpromoted).unwrap();
                }
                self.inner.xor_piece(from, last_moved);
                self.inner.xor_piece(to, self.inner.piece_at(to).unwrap());
                *self.inner.piece_at_mut(from) = Some(last_moved);
                *self.inner.piece_at_mut(to) = captured;
            }
            Move::Drop { to, piece } => {
                self.inner.xor_piece(to, piece);
                *self.inner.piece_at_mut(to) = None;
                let hand = self.inner.hand_of_a_player_mut(c);
                *hand = hand.added(piece.piece_kind()).unwrap();
            }
        }
        self.inner.side = c;
        self.inner.ply -= 1;
        self.states.pop();
    }
    #[inline(always)]
    pub(crate) fn player_bitboard(&self, c: Color) -> Bitboard {
        self.inner.player_bb[c.array_index()]
    }
    #[inline(always)]
    pub(crate) fn piece_kind_bitboard(&self, pk: PieceKind) -> Bitboard {
        self.inner.piece_bb[pk.array_index()]
    }
    #[inline(always)]
    pub(crate) fn piece_bitboard(&self, p: Piece) -> Bitboard {
        let (pk, c) = p.to_parts();
        self.inner.piece_bb[pk.array_index()] & self.inner.player_bb[c.array_index()]
    }
    #[inline(always)]
    pub(crate) fn occupied_bitboard(&self) -> Bitboard {
        self.inner.occupied_bitboard()
    }
    #[inline(always)]
    pub(crate) fn king_position(&self, c: Color) -> Option<Square> {
        self.inner.king_position(c)
    }
    #[inline(always)]
    pub(crate) fn captured(&self) -> Option<Piece> {
        self.state().captured
    }
    #[inline(always)]
    pub(crate) fn last_moved(&self) -> Option<Piece> {
        self.state().last_moved
    }
    #[inline(always)]
    pub(crate) fn checkers(&self) -> Bitboard {
        self.state().attack_info.checkers()
    }
    #[inline(always)]
    pub(crate) fn pinned(&self, c: Color) -> Bitboard {
        self.state().attack_info.pinned(c)
    }
    #[inline(always)]
    fn state(&self) -> &State {
        self.states.last().expect("empty states")
    }
    #[inline(always)]
    fn checkable(&self, pk: PieceKind, sq: Square) -> bool {
        self.state().attack_info.checkable(pk, sq)
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new(shogi_core::PartialPosition::startpos())
    }
}

/// Represents a state of a single position of a game.
#[derive(Clone, Debug)]
pub(crate) struct PartialPosition {
    side: Color,
    ply: u16,
    hands: [Hand; Color::NUM],
    board: [Option<Piece>; Square::NUM],
    player_bb: [Bitboard; Color::NUM],
    piece_bb: [Bitboard; PieceKind::NUM],
}

impl PartialPosition {
    #[inline(always)]
    fn xor_piece(&mut self, sq: Square, p: Piece) {
        let single = Bitboard::single(sq);
        let (pk, c) = p.to_parts();
        self.player_bb[c.array_index()] ^= single;
        self.piece_bb[pk.array_index()] ^= single;
    }
    #[inline(always)]
    fn piece_at(&self, sq: Square) -> Option<Piece> {
        self.board[sq.array_index()]
    }
    #[inline(always)]
    fn piece_at_mut(&mut self, sq: Square) -> &mut Option<Piece> {
        &mut self.board[sq.array_index()]
    }
    #[inline(always)]
    fn hand_of_a_player(&self, c: Color) -> Hand {
        self.hands[c.array_index()]
    }
    #[inline(always)]
    fn hand_of_a_player_mut(&mut self, c: Color) -> &mut Hand {
        &mut self.hands[c.array_index()]
    }
    #[inline(always)]
    fn occupied_bitboard(&self) -> Bitboard {
        self.player_bb[0] | self.player_bb[1]
    }
    #[inline(always)]
    fn king_position(&self, c: Color) -> Option<Square> {
        (self.player_bb[c.array_index()] & self.piece_bb[PieceKind::King.array_index()])
            .into_iter()
            .next()
    }
}

impl From<shogi_core::PartialPosition> for PartialPosition {
    fn from(pp: shogi_core::PartialPosition) -> Self {
        let mut hands = [Hand::default(); Color::NUM];
        let mut board = [None; Square::NUM];
        let mut player_bb = [Bitboard::empty(); Color::NUM];
        let mut piece_bb = [Bitboard::empty(); PieceKind::NUM];
        for c in Color::all() {
            hands[c.array_index()] = pp.hand_of_a_player(c);
        }
        for sq in Square::all() {
            let piece_at = pp.piece_at(sq);
            board[sq.array_index()] = piece_at;
            if let Some(p) = piece_at {
                player_bb[p.color().array_index()] |= Bitboard::single(sq);
                piece_bb[p.piece_kind().array_index()] |= Bitboard::single(sq);
            }
        }
        Self {
            side: pp.side_to_move(),
            ply: pp.ply(),
            hands,
            board,
            player_bb,
            piece_bb,
        }
    }
}

#[derive(Debug, Clone)]
struct State {
    /// Zobrist hashes for (board ^ side, hand)
    keys: (Key, Key),
    /// Piece captured on the last move
    captured: Option<Piece>,
    /// Last moved piece
    last_moved: Option<Piece>,
    attack_info: AttackInfo,
}

#[derive(Debug, Clone)]
struct AttackInfo {
    /// 手番側の王に対して王手をかけている相手駒の位置
    checkers: Bitboard,
    /// 各駒種が王手になり得る位置
    checkables: [Bitboard; PieceKind::NUM],
    /// Color番目の玉を飛び駒から守っている駒（Color問わず）の位置
    pinned: [Bitboard; Color::NUM],
}

impl AttackInfo {
    pub fn new(checkers: Bitboard, pos: &PartialPosition) -> Self {
        let opp = pos.side.flip();
        let occ = pos.occupied_bitboard();
        let mut pinned = [Bitboard::empty(), Bitboard::empty()];
        for c in Color::all() {
            if let Some(sq) = pos.king_position(c) {
                #[rustfmt::skip]
                let snipers = (
                      (ATTACK_TABLE.pseudo_attack(PieceKind::Lance, sq, c) & pos.piece_bb[PieceKind::Lance.array_index()])
                    | (ATTACK_TABLE.pseudo_attack(PieceKind::Bishop, sq, c) & (pos.piece_bb[PieceKind::Bishop.array_index()] | pos.piece_bb[PieceKind::ProBishop.array_index()]))
                    | (ATTACK_TABLE.pseudo_attack(PieceKind::Rook, sq, c) & (pos.piece_bb[PieceKind::Rook.array_index()] | pos.piece_bb[PieceKind::ProRook.array_index()]))
                ) & pos.player_bb[c.flip().array_index()];
                for sniper in snipers {
                    let blockers = BETWEEN_TABLE[sq.array_index()][sniper.array_index()] & occ;
                    if blockers.count() == 1 {
                        pinned[c.array_index()] |= blockers;
                    }
                }
            }
        }
        if let Some(sq) = pos.king_position(opp) {
            let ka = ATTACK_TABLE.ka.attack(sq, &occ);
            let hi = ATTACK_TABLE.hi.attack(sq, &occ);
            let ki = ATTACK_TABLE.ki.attack(sq, opp);
            let gi = ATTACK_TABLE.gi.attack(sq, opp);
            Self {
                checkers,
                checkables: [
                    ATTACK_TABLE.fu.attack(sq, opp),
                    ATTACK_TABLE.ky.attack(sq, opp, &occ),
                    ATTACK_TABLE.ke.attack(sq, opp),
                    gi,
                    ki,
                    ka,
                    hi,
                    Bitboard::empty(),
                    ki,
                    ki,
                    ki,
                    ki,
                    ka | ki,
                    hi | gi,
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
    pub fn calculate_checkers(pos: &PartialPosition) -> Bitboard {
        let c = pos.side;
        let occ = pos.occupied_bitboard();
        if let Some(sq) = pos.king_position(c) {
            (     (ATTACK_TABLE.fu.attack(sq, c)       & pos.piece_bb[PieceKind::Pawn.array_index()])
                | (ATTACK_TABLE.ky.attack(sq, c, &occ) & pos.piece_bb[PieceKind::Lance.array_index()])
                | (ATTACK_TABLE.ke.attack(sq, c)       & pos.piece_bb[PieceKind::Knight.array_index()])
                // Delta of ProRook (龍) is a superposition of GI and HI
                | (ATTACK_TABLE.gi.attack(sq, c)       & (pos.piece_bb[PieceKind::Silver.array_index()] | pos.piece_bb[PieceKind::ProRook.array_index()]))
                // Delta of ProBishop (馬) is a superposition of KA and KI
                | (ATTACK_TABLE.ka.attack(sq, &occ)    & (pos.piece_bb[PieceKind::Bishop.array_index()] | pos.piece_bb[PieceKind::ProBishop.array_index()]))
                | (ATTACK_TABLE.hi.attack(sq, &occ)    & (pos.piece_bb[PieceKind::Rook.array_index()] | pos.piece_bb[PieceKind::ProRook.array_index()]))
                | (ATTACK_TABLE.ki.attack(sq, c)       & (pos.piece_bb[PieceKind::Gold.array_index()] | pos.piece_bb[PieceKind::ProPawn.array_index()] | pos.piece_bb[PieceKind::ProLance.array_index()] | pos.piece_bb[PieceKind::ProKnight.array_index()] | pos.piece_bb[PieceKind::ProSilver.array_index()] | pos.piece_bb[PieceKind::ProBishop.array_index()]))
            ) & pos.player_bb[c.flip().array_index()]
        } else {
            Bitboard::empty()
        }
    }
    #[inline(always)]
    pub fn checkers(&self) -> Bitboard {
        self.checkers
    }
    #[inline(always)]
    pub fn pinned(&self, c: Color) -> Bitboard {
        self.pinned[c.array_index()]
    }
    #[inline(always)]
    pub fn checkable(&self, pk: PieceKind, sq: Square) -> bool {
        self.checkables[pk.array_index()].contains(sq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shogi_core::PartialPosition;
    use shogi_usi_parser::FromUsi;

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
            assert_eq!(expected, pos.piece_at(sq), "square: {:?}", sq);
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
            Move::Normal {
                from: Square::SQ_7G,
                to: Square::SQ_7F,
                promote: false,
            },
            Move::Normal {
                from: Square::SQ_3C,
                to: Square::SQ_3D,
                promote: false,
            },
            Move::Normal {
                from: Square::SQ_8H,
                to: Square::SQ_2B,
                promote: true,
            },
            Move::Normal {
                from: Square::SQ_3A,
                to: Square::SQ_2B,
                promote: false,
            },
            Move::Drop {
                to: Square::SQ_3C,
                piece: Piece::B_B,
            },
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
            assert_eq!(expected, pos.piece_at(sq), "square: {:?}", sq);
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
        assert!(Square::all().all(|sq| pos.piece_at(sq) == default.piece_at(sq)));
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
        // https://qiita.com/ak11/items/8bd5f2bb0f5b014143c8#%E5%88%9D%E6%9C%9F%E5%B1%80%E9%9D%A2%E3%81%98%E3%82%83%E3%81%AA%E3%81%84%E5%B1%80%E9%9D%A2
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
        {
            let mut pos = Position::new(
                PartialPosition::from_usi(
                    "sfen R8/2K1S1SSk/4B4/9/9/9/9/9/1L1L1L3 b RBGSNLP3g3n17p 1",
                )
                .expect("failed to parse"),
            );
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
        let pos = Position::new(
            PartialPosition::from_usi("sfen 6p1k/9/6P1G/9/8L/9/9/9/9 b RBLrb3g4s4n2l16p 1")
                .expect("failed to parse"),
        );
        let test_cases = [
            (
                Move::Drop {
                    to: Square::SQ_1B,
                    piece: Piece::B_L,
                },
                true,
            ),
            (
                Move::Drop {
                    to: Square::SQ_1D,
                    piece: Piece::B_L,
                },
                false,
            ),
            (
                Move::Drop {
                    to: Square::SQ_2B,
                    piece: Piece::B_B,
                },
                true,
            ),
            (
                Move::Drop {
                    to: Square::SQ_5E,
                    piece: Piece::B_B,
                },
                false,
            ),
            (
                Move::Drop {
                    to: Square::SQ_2A,
                    piece: Piece::B_R,
                },
                true,
            ),
            (
                Move::Drop {
                    to: Square::SQ_5A,
                    piece: Piece::B_R,
                },
                false,
            ),
            (
                Move::Normal {
                    from: Square::SQ_1C,
                    to: Square::SQ_1B,
                    promote: false,
                },
                true,
            ),
            (
                Move::Normal {
                    from: Square::SQ_1C,
                    to: Square::SQ_2B,
                    promote: false,
                },
                true,
            ),
            (
                Move::Normal {
                    from: Square::SQ_1C,
                    to: Square::SQ_2C,
                    promote: false,
                },
                true,
            ),
            (
                Move::Normal {
                    from: Square::SQ_1C,
                    to: Square::SQ_1D,
                    promote: false,
                },
                false,
            ),
        ];
        for (m, expected) in test_cases {
            assert_eq!(expected, pos.is_check_move(m));
        }
    }
}
