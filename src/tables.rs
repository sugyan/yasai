use crate::array_index::ArrayIndex;
use crate::bitboard::Bitboard;
use crate::square::{File, Rank};
use crate::Square;
use bitintr::Pext;
use once_cell::sync::Lazy;
use shogi_core::{Color, PieceKind};
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

pub struct PieceAttackTable([[Bitboard; 2]; Square::NUM]);

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

    fn new(deltas: &[&[Delta]; 2]) -> Self {
        let mut table = [[Bitboard::ZERO; 2]; Square::NUM];
        for sq in Square::ALL {
            for color in Color::all() {
                for &delta in deltas[color.array_index()] {
                    if let Some(to) = sq.checked_shift(delta.file, delta.rank) {
                        table[sq.index()][color.array_index()] |= to;
                    }
                }
            }
        }
        Self(table)
    }
    pub fn attack(&self, sq: Square, c: Color) -> Bitboard {
        self.0[sq.index()][c.array_index()]
    }
}

fn sliding_attacks(sq: Square, occ: Bitboard, deltas: &[Delta]) -> Bitboard {
    let mut bb = Bitboard::ZERO;
    for &delta in deltas {
        let mut curr = sq.checked_shift(delta.file, delta.rank);
        while let Some(to) = curr {
            bb |= to;
            if !(occ & to).is_empty() {
                break;
            }
            curr = to.checked_shift(delta.file, delta.rank);
        }
    }
    bb
}

pub struct LanceAttackTable {
    table: Vec<Bitboard>,
    offsets: [[usize; 2]; Square::NUM],
}

impl LanceAttackTable {
    const MASK_BITS: u32 = 7;
    const MASK_TABLE_NUM: usize = 1 << LanceAttackTable::MASK_BITS;
    #[rustfmt::skip]
    const OFFSET_BITS: [u32; Square::NUM] = [
         1,  1,  1,  1,  1,  1,  1,  1,  1,
        10, 10, 10, 10, 10, 10, 10, 10, 10,
        19, 19, 19, 19, 19, 19, 19, 19, 19,
        28, 28, 28, 28, 28, 28, 28, 28, 28,
        37, 37, 37, 37, 37, 37, 37, 37, 37,
        46, 46, 46, 46, 46, 46, 46, 46, 46,
        55, 55, 55, 55, 55, 55, 55, 55, 55,
         1,  1,  1,  1,  1,  1,  1,  1,  1,
        10, 10, 10, 10, 10, 10, 10, 10, 10,
    ];

    fn attack_mask(sq: Square) -> Bitboard {
        Bitboard::from_file(sq.file())
            & !(Bitboard::from_rank(Rank::RANK1) | Bitboard::from_rank(Rank::RANK9))
    }
    fn index_to_occupied(index: usize, mask: Bitboard) -> Bitboard {
        let mut bb = Bitboard::ZERO;
        for (i, sq) in mask.enumerate() {
            if (index & (1 << i)) != 0 {
                bb |= sq;
            }
        }
        bb
    }
    fn new() -> Self {
        let mut table = vec![Bitboard::ZERO; Square::NUM * 2 * LanceAttackTable::MASK_TABLE_NUM];
        let mut offsets = [[0; 2]; Square::NUM];
        let mut offset = 0;
        for sq in Square::ALL {
            let mask = Self::attack_mask(sq);
            for c in Color::all() {
                let deltas = match c {
                    Color::Black => vec![Delta::N],
                    Color::White => vec![Delta::S],
                };
                offsets[sq.index()][c.array_index()] = offset;
                for index in 0..LanceAttackTable::MASK_TABLE_NUM {
                    let occ = Self::index_to_occupied(index, mask);
                    table[offset + index] = sliding_attacks(sq, occ, &deltas);
                }
                offset += LanceAttackTable::MASK_TABLE_NUM;
            }
        }
        Self { table, offsets }
    }
    fn pseudo_attack(&self, sq: Square, c: Color) -> Bitboard {
        self.table[self.offsets[sq.index()][c.array_index()]]
    }
    pub fn attack(&self, sq: Square, c: Color, occ: &Bitboard) -> Bitboard {
        let index = ((occ.value(if sq.index() >= 63 { 1 } else { 0 })
            >> LanceAttackTable::OFFSET_BITS[sq.index()]) as usize)
            & (Self::MASK_TABLE_NUM - 1);
        self.table[self.offsets[sq.index()][c.array_index()] + index]
    }
}

pub struct SlidingAttackTable {
    table: Vec<Bitboard>,
    masks: [Bitboard; Square::NUM],
    offsets: [usize; Square::NUM],
}

impl SlidingAttackTable {
    fn attack_mask(sq: Square, deltas: &[Delta]) -> Bitboard {
        let mut bb = sliding_attacks(sq, Bitboard::ZERO, deltas);
        if sq.file() != File::FILE1 {
            bb &= !Bitboard::FILES[File::FILE1.0 as usize];
        }
        if sq.file() != File::FILE9 {
            bb &= !Bitboard::FILES[File::FILE9.0 as usize];
        }
        if sq.rank() != Rank::RANK1 {
            bb &= !Bitboard::RANKS[Rank::RANK1.0 as usize];
        }
        if sq.rank() != Rank::RANK9 {
            bb &= !Bitboard::RANKS[Rank::RANK9.0 as usize];
        }
        bb
    }
    fn index_to_occupied(index: usize, mask: Bitboard) -> Bitboard {
        let mut bb = Bitboard::ZERO;
        for (i, sq) in mask.enumerate() {
            if (index & (1 << i)) != 0 {
                bb |= sq;
            }
        }
        bb
    }
    fn occupied_to_index(occ: Bitboard, mask: Bitboard) -> usize {
        (occ & mask).merge().pext(mask.merge()) as usize
    }
    fn new(table_size: usize, deltas: &[Delta]) -> Self {
        let mut table = vec![Bitboard::ZERO; table_size];
        let mut masks = [Bitboard::ZERO; Square::NUM];
        let mut offsets = [0; Square::NUM];
        let mut offset = 0;
        for sq in Square::ALL {
            let mask = SlidingAttackTable::attack_mask(sq, deltas);
            let ones = mask.count_ones();
            masks[sq.index()] = mask;
            offsets[sq.index()] = offset;
            for index in 0..1 << ones {
                let occ = Self::index_to_occupied(index, mask);
                table[offset + Self::occupied_to_index(occ, mask)] =
                    sliding_attacks(sq, occ, deltas);
            }
            offset += 1 << ones;
        }
        Self {
            table,
            masks,
            offsets,
        }
    }
    fn pseudo_attack(&self, sq: Square) -> Bitboard {
        self.table[self.offsets[sq.index()]]
    }
    pub fn attack(&self, sq: Square, occ: &Bitboard) -> Bitboard {
        self.table[self.offsets[sq.index()] + Self::occupied_to_index(*occ, self.masks[sq.index()])]
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
    pub fn attack(&self, pk: PieceKind, sq: Square, c: Color, occ: &Bitboard) -> Bitboard {
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
    pub fn pseudo_attack(&self, pk: PieceKind, sq: Square, c: Color) -> Bitboard {
        match pk {
            PieceKind::Lance => self.ky.pseudo_attack(sq, c),
            PieceKind::Bishop | PieceKind::ProBishop => self.ka.pseudo_attack(sq),
            PieceKind::Rook | PieceKind::ProRook => self.hi.pseudo_attack(sq),
            pk => self.attack(pk, sq, c, &Bitboard::ZERO),
        }
    }
}

pub static ATTACK_TABLE: Lazy<AttackTable> = Lazy::new(|| {
    AttackTable {
        fu: PieceAttackTable::new(&[PieceAttackTable::BFU_DELTAS, PieceAttackTable::WFU_DELTAS]),
        ky: LanceAttackTable::new(),
        ke: PieceAttackTable::new(&[PieceAttackTable::BKE_DELTAS, PieceAttackTable::WKE_DELTAS]),
        gi: PieceAttackTable::new(&[PieceAttackTable::BGI_DELTAS, PieceAttackTable::WGI_DELTAS]),
        ka: SlidingAttackTable::new(
            20224, // 1 * (1 << 12) + 8 * (1 << 10) + 16 * (1 << 8) + 52 * (1 << 6) + 4 * (1 << 7)
            &[Delta::NE, Delta::SE, Delta::SW, Delta::NW],
        ),
        hi: SlidingAttackTable::new(
            495616, // 4 * (1 << 14) + 28 * (1 << 13) + 49 * (1 << 12)
            &[Delta::N, Delta::E, Delta::S, Delta::W],
        ),
        ki: PieceAttackTable::new(&[PieceAttackTable::BKI_DELTAS, PieceAttackTable::WKI_DELTAS]),
        ou: PieceAttackTable::new(&[PieceAttackTable::BOU_DELTAS, PieceAttackTable::WOU_DELTAS]),
    }
});

pub static BETWEEN_TABLE: Lazy<[[Bitboard; Square::NUM]; Square::NUM]> = Lazy::new(|| {
    let mut bbs = [[Bitboard::ZERO; Square::NUM]; Square::NUM];
    for sq0 in Square::ALL {
        for sq1 in Square::ALL {
            let (df, dr) = (sq1.file() - sq0.file(), sq1.rank() - sq0.rank());
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
            bbs[sq0.index()][sq1.index()] =
                sliding_attacks(sq0, Bitboard::from_square(sq1), &[delta])
                    & !Bitboard::from_square(sq1);
        }
    }
    bbs
});
