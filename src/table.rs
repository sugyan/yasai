use crate::bitboard::Bitboard;
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
    const BGI_DELTAS: &'static [Delta] = &[Delta::N, Delta::NE, Delta::SE, Delta::SW, Delta::NW];
    const WGI_DELTAS: &'static [Delta] = &[Delta::S, Delta::NE, Delta::SE, Delta::SW, Delta::NW];
    #[rustfmt::skip]
    const BKI_DELTAS: &'static [Delta] = &[Delta::N, Delta::E, Delta::S, Delta::W, Delta::NE, Delta::NW];
    #[rustfmt::skip]
    const WKI_DELTAS: &'static [Delta] = &[Delta::N, Delta::E, Delta::S, Delta::W, Delta::SE, Delta::SW];

    fn new(deltas: &[&[Delta]; Color::NUM]) -> Self {
        let mut table = [[Bitboard::ZERO; Color::NUM]; Square::NUM];
        for color in &Color::ALL {
            for &sq in &Square::ALL {
                for &delta in deltas[color.0 as usize] {
                    if let Some(to) = sq + delta {
                        if (sq.file() - to.file()).abs() <= 1 && (sq.rank() - to.rank()).abs() <= 2
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

pub struct AttackTable {
    pub gi: PieceAttackTable,
    pub ki: PieceAttackTable,
}

impl AttackTable {}

pub static ATTACK_TABLE: Lazy<AttackTable> = Lazy::new(|| AttackTable {
    gi: PieceAttackTable::new(&[PieceAttackTable::BGI_DELTAS, PieceAttackTable::WGI_DELTAS]),
    ki: PieceAttackTable::new(&[PieceAttackTable::BKI_DELTAS, PieceAttackTable::WKI_DELTAS]),
});
