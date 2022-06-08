use crate::tables::{ATTACK_TABLE, BETWEEN_TABLE};
use crate::zobrist::Key;
use shogi_core::{Bitboard, Color, PartialPosition, Piece, PieceKind, Square};

#[derive(Debug, Clone)]
pub(crate) struct AttackInfo {
    checkers: Bitboard,                     // 王手をかけている駒の位置
    checkables: [Bitboard; PieceKind::NUM], // 各駒種が王手になり得る位置
    pinned: [Bitboard; 2],                  // 飛び駒から玉を守っている駒の位置
}

impl AttackInfo {
    pub fn new(checkers: Bitboard, pos: &PartialPosition) -> Self {
        let occ = &!pos.vacant_bitboard();
        let opp = pos.side_to_move().flip();
        let mut pinned = [Bitboard::empty(), Bitboard::empty()];
        for c in Color::all() {
            if let Some(sq) = pos.king_position(c) {
                #[rustfmt::skip]
                let snipers = (
                      (ATTACK_TABLE.pseudo_attack(PieceKind::Lance, sq, c) & pos.piece_kind_bitboard(PieceKind::Lance))
                    | (ATTACK_TABLE.pseudo_attack(PieceKind::Bishop, sq, c) & (pos.piece_kind_bitboard(PieceKind::Bishop) | pos.piece_kind_bitboard(PieceKind::ProBishop)))
                    | (ATTACK_TABLE.pseudo_attack(PieceKind::Rook, sq, c) & (pos.piece_kind_bitboard(PieceKind::Rook) | pos.piece_kind_bitboard(PieceKind::ProRook)))
                ) & pos.player_bitboard(c.flip());
                for sniper in snipers {
                    let blockers = BETWEEN_TABLE[sq.array_index()][sniper.array_index()] & *occ;
                    if blockers.count() == 1 {
                        pinned[c.array_index()] |= blockers;
                    }
                }
            }
        }
        if let Some(sq) = pos.king_position(opp) {
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
    pub fn calculate_checkers(pos: &PartialPosition) -> Bitboard {
        let c = pos.side_to_move();
        let occ = &!pos.vacant_bitboard();
        if let Some(sq) = pos.king_position(c) {
            (     (ATTACK_TABLE.fu.attack(sq, c)      & pos.piece_kind_bitboard(PieceKind::Pawn))
                | (ATTACK_TABLE.ky.attack(sq, c, occ) & pos.piece_kind_bitboard(PieceKind::Lance))
                | (ATTACK_TABLE.ke.attack(sq, c)      & pos.piece_kind_bitboard(PieceKind::Knight))
                | (ATTACK_TABLE.gi.attack(sq, c)      & (pos.piece_kind_bitboard(PieceKind::Silver) | pos.piece_kind_bitboard(PieceKind::ProRook)))
                | (ATTACK_TABLE.ka.attack(sq, occ)    & (pos.piece_kind_bitboard(PieceKind::Bishop) | pos.piece_kind_bitboard(PieceKind::ProBishop)))
                | (ATTACK_TABLE.hi.attack(sq, occ)    & (pos.piece_kind_bitboard(PieceKind::Rook) | pos.piece_kind_bitboard(PieceKind::ProRook)))
                | (ATTACK_TABLE.ki.attack(sq, c)      & (pos.piece_kind_bitboard(PieceKind::Gold) | pos.piece_kind_bitboard(PieceKind::ProPawn) | pos.piece_kind_bitboard(PieceKind::ProLance) | pos.piece_kind_bitboard(PieceKind::ProKnight) | pos.piece_kind_bitboard(PieceKind::ProSilver) | pos.piece_kind_bitboard(PieceKind::ProBishop)))
            ) & pos.player_bitboard(c.flip())
        } else {
            Bitboard::empty()
        }
    }
    pub fn checkers(&self) -> Bitboard {
        self.checkers
    }
    pub fn pinned(&self, c: Color) -> Bitboard {
        self.pinned[c.array_index()]
    }
    pub fn checkable(&self, pk: PieceKind, sq: Square) -> bool {
        self.checkables[pk.array_index()].contains(sq)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct State {
    keys: (Key, Key),
    captured: Option<Piece>,
    last_moved: Option<Piece>,
    attack_info: AttackInfo,
}

impl State {
    pub fn new(
        keys: (Key, Key),
        captured: Option<Piece>,
        last_moved: Option<Piece>,
        attack_info: AttackInfo,
    ) -> Self {
        Self {
            keys,
            captured,
            last_moved,
            attack_info,
        }
    }
    pub fn key(&self) -> Key {
        self.keys.0 ^ self.keys.1
    }
    pub fn keys(&self) -> (Key, Key) {
        self.keys
    }
    pub fn captured(&self) -> Option<Piece> {
        self.captured
    }
    pub fn last_moved(&self) -> Option<Piece> {
        self.last_moved
    }
    pub fn attack_info(&self) -> &AttackInfo {
        &self.attack_info
    }
}
