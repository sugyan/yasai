use crate::bitboard::Bitboard;
use crate::square::Rank;
use crate::{Color, Square};
use once_cell::sync::Lazy;

#[derive(Clone, Copy)]
struct Delta(pub i8);

impl Delta {
    const N: Delta = Delta(-1);
    const E: Delta = Delta(-9);
    const S: Delta = Delta(1);
    const W: Delta = Delta(9);
    const NE: Delta = Delta(Delta::N.0 + Delta::E.0);
    const SE: Delta = Delta(Delta::S.0 + Delta::E.0);
    const SW: Delta = Delta(Delta::S.0 + Delta::W.0);
    const NW: Delta = Delta(Delta::N.0 + Delta::W.0);
    const NNE: Delta = Delta(Delta::N.0 + Delta::NE.0);
    const NNW: Delta = Delta(Delta::N.0 + Delta::NW.0);
    const SSE: Delta = Delta(Delta::S.0 + Delta::SE.0);
    const SSW: Delta = Delta(Delta::S.0 + Delta::SW.0);
}

impl std::ops::Add<Delta> for Square {
    type Output = Option<Square>;

    fn add(self, rhs: Delta) -> Self::Output {
        let i = self.0 + rhs.0;
        if (0..Square::NUM as i8).contains(&i) {
            Some(Square(i))
        } else {
            None
        }
    }
}

pub struct PieceAttackTable([[Bitboard; Color::NUM]; Square::NUM]);

impl PieceAttackTable {
    const BFU_DELTAS: &'static [Delta] = &[Delta::N];
    const WFU_DELTAS: &'static [Delta] = &[Delta::S];
    const BKE_DELTAS: &'static [Delta] = &[Delta::NNE, Delta::NNW];
    const WKE_DELTAS: &'static [Delta] = &[Delta::SSE, Delta::SSW];
    const BGI_DELTAS: &'static [Delta] = &[Delta::N, Delta::NE, Delta::SE, Delta::SW, Delta::NW];
    const WGI_DELTAS: &'static [Delta] = &[Delta::S, Delta::NE, Delta::SE, Delta::SW, Delta::NW];
    #[rustfmt::skip]
    const BKI_DELTAS: &'static [Delta] = &[Delta::N, Delta::E, Delta::S, Delta::W, Delta::NE, Delta::NW];
    #[rustfmt::skip]
    const WKI_DELTAS: &'static [Delta] = &[Delta::N, Delta::E, Delta::S, Delta::W, Delta::SE, Delta::SW];

    fn new(deltas: &[&[Delta]; Color::NUM]) -> Self {
        let mut table = [[Bitboard::ZERO; Color::NUM]; Square::NUM];
        for &sq in &Square::ALL {
            for &color in &Color::ALL {
                for &delta in deltas[color.0 as usize] {
                    if let Some(to) = sq + delta {
                        if (to.file() - sq.file()).abs() <= 1 && (to.rank() - sq.rank()).abs() <= 2
                        {
                            table[sq.0 as usize][color.0 as usize] |= to;
                        }
                    }
                }
            }
        }
        Self(table)
    }
    pub fn attack(&self, sq: Square, color: Color) -> Bitboard {
        self.0[sq.0 as usize][color.0 as usize]
    }
}

fn sliding_attacks(sq: Square, occupied: Bitboard, deltas: &[Delta]) -> Bitboard {
    let mut bb = Bitboard::ZERO;
    for &delta in deltas {
        let (mut prev, mut curr) = (sq, sq + delta);
        while let Some(to) = curr {
            if (to.file() - prev.file()).abs() > 1 || (to.rank() - prev.rank()).abs() > 1 {
                break;
            }
            bb |= to;
            if !(occupied & to).is_empty() {
                break;
            }
            prev = to;
            curr = to + delta;
        }
    }
    bb
}

pub struct LanceAttackTable(
    [[[Bitboard; LanceAttackTable::MASK_TABLE_NUM as usize]; Color::NUM]; Square::NUM],
);

impl LanceAttackTable {
    const MASK_BITS: u32 = 7;
    const MASK_TABLE_NUM: usize = 1 << LanceAttackTable::MASK_BITS;
    #[rustfmt::skip]
    const OFFSETS: [u32; Square::NUM] = [
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

    fn new() -> LanceAttackTable {
        let mut table = [[[Bitboard::ZERO; LanceAttackTable::MASK_TABLE_NUM as usize]; Color::NUM];
            Square::NUM];
        for &sq in &Square::ALL {
            let file_mask = Bitboard::from_file(sq.file())
                & !(Bitboard::from_rank(Rank::RANK1) | Bitboard::from_rank(Rank::RANK9));
            for &color in &Color::ALL {
                let deltas = match color {
                    Color::BLACK => vec![Delta::N],
                    Color::WHITE => vec![Delta::S],
                    _ => unreachable!(),
                };
                for index in 0..LanceAttackTable::MASK_TABLE_NUM {
                    let occupied = file_mask.enumerate().fold(Bitboard::ZERO, |acc, (i, sq)| {
                        if (index & (1 << i)) != 0 {
                            acc | sq
                        } else {
                            acc
                        }
                    });
                    table[sq.0 as usize][color.0 as usize][index] =
                        sliding_attacks(sq, occupied, &deltas);
                }
            }
        }
        Self(table)
    }
    pub fn attack(&self, sq: Square, color: Color, occupied: &Bitboard) -> Bitboard {
        let index = ((occupied.value(if sq.0 > Square::SQ79.0 { 1 } else { 0 })
            >> LanceAttackTable::OFFSETS[sq.0 as usize]) as usize)
            & (Self::MASK_TABLE_NUM - 1);
        self.0[sq.0 as usize][color.0 as usize][index]
    }
}

pub struct AttackTable {
    pub fu: PieceAttackTable,
    pub ky: LanceAttackTable,
    pub ke: PieceAttackTable,
    pub gi: PieceAttackTable,
    pub ki: PieceAttackTable,
}

impl AttackTable {}

pub static ATTACK_TABLE: Lazy<AttackTable> = Lazy::new(|| AttackTable {
    fu: PieceAttackTable::new(&[PieceAttackTable::BFU_DELTAS, PieceAttackTable::WFU_DELTAS]),
    ky: LanceAttackTable::new(),
    ke: PieceAttackTable::new(&[PieceAttackTable::BKE_DELTAS, PieceAttackTable::WKE_DELTAS]),
    gi: PieceAttackTable::new(&[PieceAttackTable::BGI_DELTAS, PieceAttackTable::WGI_DELTAS]),
    ki: PieceAttackTable::new(&[PieceAttackTable::BKI_DELTAS, PieceAttackTable::WKI_DELTAS]),
});
