use crate::bitboard::{Bitboard, Occupied};
use once_cell::sync::Lazy;
use shogi_core::{Color, PieceKind, Square};
use std::cmp::Ordering;

#[derive(Clone, Copy)]
struct Delta {
    file: i8,
    rank: i8,
}

#[rustfmt::skip]
impl Delta {
    const N:   Delta = Delta { file:  0, rank: -1 };
    const E:   Delta = Delta { file: -1, rank:  0 };
    const S:   Delta = Delta { file:  0, rank:  1 };
    const W:   Delta = Delta { file:  1, rank:  0 };
    const NE:  Delta = Delta { file: -1, rank: -1 };
    const SE:  Delta = Delta { file: -1, rank:  1 };
    const SW:  Delta = Delta { file:  1, rank:  1 };
    const NW:  Delta = Delta { file:  1, rank: -1 };
    const NNE: Delta = Delta { file: -1, rank: -2 };
    const NNW: Delta = Delta { file:  1, rank: -2 };
    const SSE: Delta = Delta { file: -1, rank:  2 };
    const SSW: Delta = Delta { file:  1, rank:  2 };
}

pub struct PieceAttackTable([[Bitboard; Color::NUM]; Square::NUM]);

impl PieceAttackTable {
    #[rustfmt::skip]    const BFU_DELTAS: &'static [Delta] = &[Delta::N];
    #[rustfmt::skip]    const WFU_DELTAS: &'static [Delta] = &[Delta::S];
    #[rustfmt::skip]    const BKE_DELTAS: &'static [Delta] = &[Delta::NNE, Delta::NNW];
    #[rustfmt::skip]    const WKE_DELTAS: &'static [Delta] = &[Delta::SSE, Delta::SSW];
    #[rustfmt::skip]    const BGI_DELTAS: &'static [Delta] = &[Delta::N, Delta::NE, Delta::SE, Delta::SW, Delta::NW];
    #[rustfmt::skip]    const WGI_DELTAS: &'static [Delta] = &[Delta::S, Delta::NE, Delta::SE, Delta::SW, Delta::NW];
    #[rustfmt::skip]    const BKI_DELTAS: &'static [Delta] = &[Delta::N, Delta::E, Delta::S, Delta::W, Delta::NE, Delta::NW];
    #[rustfmt::skip]    const WKI_DELTAS: &'static [Delta] = &[Delta::N, Delta::E, Delta::S, Delta::W, Delta::SE, Delta::SW];
    #[rustfmt::skip]    const BOU_DELTAS: &'static [Delta] = &[Delta::N, Delta::E, Delta::S, Delta::W, Delta::NE, Delta::SE, Delta::SW, Delta::NW];
    #[rustfmt::skip]    const WOU_DELTAS: &'static [Delta] = &[Delta::N, Delta::E, Delta::S, Delta::W, Delta::NE, Delta::SE, Delta::SW, Delta::NW];

    fn new(deltas: &[&[Delta]; Color::NUM]) -> Self {
        let mut table = [[Bitboard::empty(); Color::NUM]; Square::NUM];
        for sq in Square::all() {
            for color in Color::all() {
                for &delta in deltas[color.array_index()] {
                    if let Some(to) = sq.shift(delta.file, delta.rank) {
                        table[sq.array_index()][color.array_index()] |= Bitboard::single(to);
                    }
                }
            }
        }
        Self(table)
    }
    #[inline(always)]
    pub(crate) fn attack(&self, sq: Square, c: Color) -> Bitboard {
        self.0[sq.array_index()][c.array_index()]
    }
}

/// Sliding attack, potentially including a square occupied by a friend.
fn sliding_attack(sq: Square, occ: Bitboard, delta: Delta) -> Bitboard {
    let mut bb = Bitboard::empty();
    let mut curr = sq.shift(delta.file, delta.rank);
    while let Some(to) = curr {
        bb |= Bitboard::single(to);
        if occ.contains(to) {
            break;
        }
        curr = to.shift(delta.file, delta.rank);
    }
    bb
}

pub struct LanceAttackTable {
    masks: [[Bitboard; Color::NUM]; Square::NUM],
}

impl LanceAttackTable {
    fn new() -> Self {
        let mut masks = [[Bitboard::empty(); Color::NUM]; Square::NUM];
        for sq in Square::all() {
            for c in Color::all() {
                let delta = match c {
                    Color::Black => Delta::N,
                    Color::White => Delta::S,
                };
                masks[sq.array_index()][c.array_index()] =
                    sliding_attack(sq, Bitboard::empty(), delta);
            }
        }
        Self { masks }
    }
    /// Attack as if there were nothing else on the board.
    #[inline(always)]
    fn pseudo_attack(&self, sq: Square, c: Color) -> Bitboard {
        self.masks[sq.array_index()][c.array_index()]
    }
    #[inline(always)]
    pub(crate) fn attack(&self, sq: Square, c: Color, occ: &Bitboard) -> Bitboard {
        let mask = self.masks[sq.array_index()][c.array_index()];
        match c {
            Color::Black => occ.sliding_negative_consecutive(&mask),
            Color::White => occ.sliding_positive_consecutive(&mask),
        }
    }
}

pub struct SlidingAttackTable {
    masks: [[[Bitboard; 2]; 2]; Square::NUM],
    merged_masks: [Bitboard; Square::NUM],
}

impl SlidingAttackTable {
    fn new(deltas: [[Delta; 2]; 2]) -> Self {
        let mut masks = [[[Bitboard::empty(); 2]; 2]; Square::NUM];
        let mut merged_masks = [Bitboard::empty(); Square::NUM];
        for sq in Square::all() {
            let mut merged_mask = Bitboard::empty();
            for (i, ds) in deltas.iter().enumerate() {
                for (j, &d) in ds.iter().enumerate() {
                    masks[sq.array_index()][i][j] = sliding_attack(sq, Bitboard::empty(), d);
                    merged_mask |= masks[sq.array_index()][i][j];
                }
            }
            merged_masks[sq.array_index()] = merged_mask;
        }
        Self {
            masks,
            merged_masks,
        }
    }
    /// Attack as if there were nothing else on the board.
    #[inline(always)]
    fn pseudo_attack(&self, sq: Square) -> Bitboard {
        self.merged_masks[sq.array_index()]
    }
    #[inline(always)]
    pub(crate) fn attack(&self, sq: Square, occ: &Bitboard) -> Bitboard {
        let masks = self.masks[sq.array_index()];
        occ.sliding_negatives(&masks[0]) | occ.sliding_positives(&masks[1])
    }
}

pub struct AttackTable {
    pub fu: PieceAttackTable,
    pub ky: LanceAttackTable,
    pub ke: PieceAttackTable,
    pub gi: PieceAttackTable,
    pub ka: SlidingAttackTable,
    pub hi: SlidingAttackTable,
    pub ki: PieceAttackTable,
    pub ou: PieceAttackTable,
}

impl AttackTable {
    pub(crate) fn attack(&self, pk: PieceKind, sq: Square, c: Color, occ: &Bitboard) -> Bitboard {
        match pk {
            PieceKind::Pawn => self.fu.attack(sq, c),
            PieceKind::Lance => self.ky.attack(sq, c, occ),
            PieceKind::Knight => self.ke.attack(sq, c),
            PieceKind::Silver => self.gi.attack(sq, c),
            PieceKind::Bishop => self.ka.attack(sq, occ),
            PieceKind::Rook => self.hi.attack(sq, occ),
            PieceKind::Gold
            | PieceKind::ProPawn
            | PieceKind::ProLance
            | PieceKind::ProKnight
            | PieceKind::ProSilver => self.ki.attack(sq, c),
            PieceKind::King => self.ou.attack(sq, c),
            PieceKind::ProBishop => self.ka.attack(sq, occ) | self.ou.attack(sq, c),
            PieceKind::ProRook => self.hi.attack(sq, occ) | self.ou.attack(sq, c),
        }
    }
    /// Attack as if there were nothing else on the board.
    pub(crate) fn pseudo_attack(&self, pk: PieceKind, sq: Square, c: Color) -> Bitboard {
        match pk {
            PieceKind::Lance => self.ky.pseudo_attack(sq, c),
            PieceKind::Bishop | PieceKind::ProBishop => self.ka.pseudo_attack(sq),
            PieceKind::Rook | PieceKind::ProRook => self.hi.pseudo_attack(sq),
            pk => self.attack(pk, sq, c, &Bitboard::empty()),
        }
    }
}

pub static ATTACK_TABLE: Lazy<AttackTable> = Lazy::new(|| AttackTable {
    fu: PieceAttackTable::new(&[PieceAttackTable::BFU_DELTAS, PieceAttackTable::WFU_DELTAS]),
    ky: LanceAttackTable::new(),
    ke: PieceAttackTable::new(&[PieceAttackTable::BKE_DELTAS, PieceAttackTable::WKE_DELTAS]),
    gi: PieceAttackTable::new(&[PieceAttackTable::BGI_DELTAS, PieceAttackTable::WGI_DELTAS]),
    ka: SlidingAttackTable::new([[Delta::NE, Delta::SE], [Delta::SW, Delta::NW]]),
    hi: SlidingAttackTable::new([[Delta::N, Delta::E], [Delta::S, Delta::W]]),
    ki: PieceAttackTable::new(&[PieceAttackTable::BKI_DELTAS, PieceAttackTable::WKI_DELTAS]),
    ou: PieceAttackTable::new(&[PieceAttackTable::BOU_DELTAS, PieceAttackTable::WOU_DELTAS]),
});

/// A table of the squares between two squares. "Between" is defined only for 8 directions.
pub(crate) static BETWEEN_TABLE: Lazy<[[Bitboard; Square::NUM]; Square::NUM]> = Lazy::new(|| {
    let mut bbs = [[Bitboard::empty(); Square::NUM]; Square::NUM];
    for sq0 in Square::all() {
        for sq1 in Square::all() {
            let (df, dr) = (
                sq1.file() as i8 - sq0.file() as i8,
                sq1.rank() as i8 - sq0.rank() as i8,
            );
            if (df | dr == 0) || (df != 0 && dr != 0 && df.abs() != dr.abs()) {
                continue;
            }
            #[rustfmt::skip]
            let delta = match (df.cmp(&0), dr.cmp(&0)) {
                (Ordering::Equal,   Ordering::Less)    => Delta::N,
                (Ordering::Less,    Ordering::Equal)   => Delta::E,
                (Ordering::Equal,   Ordering::Greater) => Delta::S,
                (Ordering::Greater, Ordering::Equal)   => Delta::W,
                (Ordering::Less,    Ordering::Less)    => Delta::NE,
                (Ordering::Less,    Ordering::Greater) => Delta::SE,
                (Ordering::Greater, Ordering::Greater) => Delta::SW,
                (Ordering::Greater, Ordering::Less)    => Delta::NW,
                _ => unreachable!(),
            };
            bbs[sq0.array_index()][sq1.array_index()] =
                sliding_attack(sq0, Bitboard::single(sq1), delta) & !Bitboard::single(sq1);
        }
    }
    bbs
});

pub(crate) static RELATIVE_RANKS: Lazy<[[usize; Color::NUM]; Square::NUM]> = Lazy::new(|| {
    let mut ranks = [[0; Color::NUM]; Square::NUM];
    for sq in Square::all() {
        for c in Color::all() {
            ranks[sq.array_index()][c.array_index()] = sq.relative_rank(c) as usize;
        }
    }
    ranks
});

pub(crate) static PROMOTABLE: Lazy<[[bool; Color::NUM]; Square::NUM]> = Lazy::new(|| {
    let mut table = [[false; Color::NUM]; Square::NUM];
    for sq in Square::all() {
        for c in Color::all() {
            if RELATIVE_RANKS[sq.array_index()][c.array_index()] <= 3 {
                table[sq.array_index()][c.array_index()] = true;
            }
        }
    }
    table
});

pub(crate) static FILES: Lazy<[Bitboard; 10]> = Lazy::new(|| {
    let mut bbs = [Bitboard::empty(); 10];
    for sq in Square::all() {
        bbs[usize::from(sq.file())] |= Bitboard::single(sq);
    }
    bbs
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Position;
    use shogi_core::consts::square::*;
    use shogi_core::PartialPosition;
    use shogi_usi_parser::FromUsi;

    #[test]
    fn lance_attack() {
        {
            let pos = Position::new(PartialPosition::startpos());
            let test_cases = [
                (SQ_1I, Color::Black, vec![SQ_1G, SQ_1H]),
                (SQ_9I, Color::Black, vec![SQ_9G, SQ_9H]),
                (SQ_1A, Color::White, vec![SQ_1B, SQ_1C]),
                (SQ_9A, Color::White, vec![SQ_9B, SQ_9C]),
            ];
            for (sq, c, expected) in test_cases {
                let bb = ATTACK_TABLE.ky.attack(sq, c, &pos.occupied_bitboard());
                assert_eq!(expected, bb.into_iter().collect::<Vec<_>>());
            }
        }
        {
            // P1-FU *  *  *  *  *  *  *  *
            // P2 * -FU *  *  *  *  *  *  *
            // P3 *  * -FU *  *  *  *  *  *
            // P4 *  *  * -FU *  *  *  *  *
            // P5 *  *  *  *  *  *  *  *  *
            // P6 *  *  * +KY *  *  *  *  *
            // P7 *  * +KY *  *  *  *  *  *
            // P8 * +KY *  *  *  *  *  *  *
            // P9+KY *  *  *  *  *  *  *  *
            // +
            let pos = Position::new(
                PartialPosition::from_usi("sfen p8/1p7/2p6/3p5/9/3L5/2L6/1L7/L8 b - 1")
                    .expect("failed to parse"),
            );
            let test_cases = [
                (
                    SQ_9I,
                    vec![SQ_9A, SQ_9B, SQ_9C, SQ_9D, SQ_9E, SQ_9F, SQ_9G, SQ_9H],
                ),
                (SQ_8H, vec![SQ_8B, SQ_8C, SQ_8D, SQ_8E, SQ_8F, SQ_8G]),
                (SQ_7G, vec![SQ_7C, SQ_7D, SQ_7E, SQ_7F]),
                (SQ_6F, vec![SQ_6D, SQ_6E]),
            ];
            for (sq, expected) in test_cases {
                let bb = ATTACK_TABLE
                    .ky
                    .attack(sq, Color::Black, &pos.occupied_bitboard());
                assert_eq!(expected, bb.into_iter().collect::<Vec<_>>());
            }
        }
        {
            // P1-KY *  *  *  *  *  *  *  *
            // P2 * -KY *  *  *  *  *  *  *
            // P3 *  * -KY *  *  *  *  *  *
            // P4 *  *  * -KY *  *  *  *  *
            // P5 *  *  *  *  *  *  *  *  *
            // P6 *  *  * +FU *  *  *  *  *
            // P7 *  * +FU *  *  *  *  *  *
            // P8 * +FU *  *  *  *  *  *  *
            // P9+FU *  *  *  *  *  *  *  *
            // +
            let pos = Position::new(
                PartialPosition::from_usi("sfen l8/1l7/2l6/3l5/9/3P5/2P6/1P7/P8 w - 1")
                    .expect("failed to parse"),
            );
            let test_cases = [
                (
                    SQ_9A,
                    vec![SQ_9B, SQ_9C, SQ_9D, SQ_9E, SQ_9F, SQ_9G, SQ_9H, SQ_9I],
                ),
                (SQ_8B, vec![SQ_8C, SQ_8D, SQ_8E, SQ_8F, SQ_8G, SQ_8H]),
                (SQ_7C, vec![SQ_7D, SQ_7E, SQ_7F, SQ_7G]),
                (SQ_6D, vec![SQ_6E, SQ_6F]),
            ];
            for (sq, expected) in test_cases {
                let bb = ATTACK_TABLE
                    .ky
                    .attack(sq, Color::White, &pos.occupied_bitboard());
                assert_eq!(expected, bb.into_iter().collect::<Vec<_>>());
            }
        }
    }

    #[test]
    fn sliding_attack() {
        {
            let pos = Position::new(PartialPosition::startpos());
            {
                let test_cases = [
                    (SQ_2B, vec![SQ_1A, SQ_1C, SQ_3A, SQ_3C]),
                    (SQ_8H, vec![SQ_7G, SQ_7I, SQ_9G, SQ_9I]),
                ];
                for (sq, expected) in test_cases {
                    let bb = ATTACK_TABLE.ka.attack(sq, &pos.occupied_bitboard());
                    assert_eq!(expected, bb.into_iter().collect::<Vec<_>>());
                }
            }
            {
                let test_cases = [
                    (
                        SQ_2H,
                        vec![
                            SQ_1H, SQ_2G, SQ_2I, SQ_3H, SQ_4H, SQ_5H, SQ_6H, SQ_7H, SQ_8H,
                        ],
                    ),
                    (
                        SQ_8B,
                        vec![
                            SQ_2B, SQ_3B, SQ_4B, SQ_5B, SQ_6B, SQ_7B, SQ_8A, SQ_8C, SQ_9B,
                        ],
                    ),
                ];
                for (sq, expected) in test_cases {
                    let bb = ATTACK_TABLE.hi.attack(sq, &pos.occupied_bitboard());
                    assert_eq!(expected, bb.into_iter().collect::<Vec<_>>());
                }
            }
        }
    }
}
