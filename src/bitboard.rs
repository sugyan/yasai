use crate::square::{File, Rank};
use crate::Square;
use std::{fmt, ops};

#[derive(Clone, Copy)]
pub union Bitboard {
    u: [u64; 2],
}

impl Bitboard {
    #[rustfmt::skip]    pub const ZERO: Bitboard = Bitboard { u: [                    0,           0] };
    #[rustfmt::skip]    pub const ONES: Bitboard = Bitboard { u: [0x7fff_ffff_ffff_ffff, 0x0003_ffff] };

    pub fn is_empty(&self) -> bool {
        (self.value(0) | self.value(1)) == 0
    }
    pub fn count_ones(&self) -> u32 {
        self.value(0).count_ones() + self.value(1).count_ones()
    }
    pub fn pop(&mut self) -> Option<Square> {
        if self.value(0) != 0 {
            Some(self.pop0())
        } else if self.value(1) != 0 {
            Some(self.pop1())
        } else {
            None
        }
    }
    pub fn value(&self, i: usize) -> u64 {
        unsafe { self.u[i] }
    }
    pub fn from_square(sq: Square) -> Bitboard {
        Bitboard::SQUARE[sq.index()]
    }
    pub fn from_file(file: File) -> Bitboard {
        Bitboard::FILES[file.0 as usize]
    }
    pub fn from_rank(rank: Rank) -> Bitboard {
        Bitboard::RANKS[rank.0 as usize]
    }
    pub fn merge(&self) -> u64 {
        self.value(0) | self.value(1)
    }
    fn pop0(&mut self) -> Square {
        let sq = Square(self.value(0).trailing_zeros() as i8);
        unsafe {
            self.u[0] &= self.u[0] - 1;
        }
        sq
    }
    fn pop1(&mut self) -> Square {
        let sq = Square(self.value(1).trailing_zeros() as i8 + 63);
        unsafe {
            self.u[1] &= self.u[1] - 1;
        }
        sq
    }

    #[rustfmt::skip]
    pub const FILES: [Bitboard; File::NUM] = [
        Bitboard { u: [0x01ff      ,            0] },
        Bitboard { u: [0x01ff <<  9,            0] },
        Bitboard { u: [0x01ff << 18,            0] },
        Bitboard { u: [0x01ff << 27,            0] },
        Bitboard { u: [0x01ff << 36,            0] },
        Bitboard { u: [0x01ff << 45,            0] },
        Bitboard { u: [0x01ff << 54,            0] },
        Bitboard { u: [           0, 0x01ff      ] },
        Bitboard { u: [           0, 0x01ff <<  9] }
    ];
    #[rustfmt::skip]
    pub const RANKS: [Bitboard; Rank::NUM] = [
        Bitboard { u: [0x0040_2010_0804_0201     , 0x0201     ] },
        Bitboard { u: [0x0040_2010_0804_0201 << 1, 0x0201 << 1] },
        Bitboard { u: [0x0040_2010_0804_0201 << 2, 0x0201 << 2] },
        Bitboard { u: [0x0040_2010_0804_0201 << 3, 0x0201 << 3] },
        Bitboard { u: [0x0040_2010_0804_0201 << 4, 0x0201 << 4] },
        Bitboard { u: [0x0040_2010_0804_0201 << 5, 0x0201 << 5] },
        Bitboard { u: [0x0040_2010_0804_0201 << 6, 0x0201 << 6] },
        Bitboard { u: [0x0040_2010_0804_0201 << 7, 0x0201 << 7] },
        Bitboard { u: [0x0040_2010_0804_0201 << 8, 0x0201 << 8] },
    ];
    #[rustfmt::skip]
    const SQUARE: [Bitboard; Square::NUM] = [
        Bitboard { u: [1 <<  0,       0] },
        Bitboard { u: [1 <<  1,       0] },
        Bitboard { u: [1 <<  2,       0] },
        Bitboard { u: [1 <<  3,       0] },
        Bitboard { u: [1 <<  4,       0] },
        Bitboard { u: [1 <<  5,       0] },
        Bitboard { u: [1 <<  6,       0] },
        Bitboard { u: [1 <<  7,       0] },
        Bitboard { u: [1 <<  8,       0] },
        Bitboard { u: [1 <<  9,       0] },
        Bitboard { u: [1 << 10,       0] },
        Bitboard { u: [1 << 11,       0] },
        Bitboard { u: [1 << 12,       0] },
        Bitboard { u: [1 << 13,       0] },
        Bitboard { u: [1 << 14,       0] },
        Bitboard { u: [1 << 15,       0] },
        Bitboard { u: [1 << 16,       0] },
        Bitboard { u: [1 << 17,       0] },
        Bitboard { u: [1 << 18,       0] },
        Bitboard { u: [1 << 19,       0] },
        Bitboard { u: [1 << 20,       0] },
        Bitboard { u: [1 << 21,       0] },
        Bitboard { u: [1 << 22,       0] },
        Bitboard { u: [1 << 23,       0] },
        Bitboard { u: [1 << 24,       0] },
        Bitboard { u: [1 << 25,       0] },
        Bitboard { u: [1 << 26,       0] },
        Bitboard { u: [1 << 27,       0] },
        Bitboard { u: [1 << 28,       0] },
        Bitboard { u: [1 << 29,       0] },
        Bitboard { u: [1 << 30,       0] },
        Bitboard { u: [1 << 31,       0] },
        Bitboard { u: [1 << 32,       0] },
        Bitboard { u: [1 << 33,       0] },
        Bitboard { u: [1 << 34,       0] },
        Bitboard { u: [1 << 35,       0] },
        Bitboard { u: [1 << 36,       0] },
        Bitboard { u: [1 << 37,       0] },
        Bitboard { u: [1 << 38,       0] },
        Bitboard { u: [1 << 39,       0] },
        Bitboard { u: [1 << 40,       0] },
        Bitboard { u: [1 << 41,       0] },
        Bitboard { u: [1 << 42,       0] },
        Bitboard { u: [1 << 43,       0] },
        Bitboard { u: [1 << 44,       0] },
        Bitboard { u: [1 << 45,       0] },
        Bitboard { u: [1 << 46,       0] },
        Bitboard { u: [1 << 47,       0] },
        Bitboard { u: [1 << 48,       0] },
        Bitboard { u: [1 << 49,       0] },
        Bitboard { u: [1 << 50,       0] },
        Bitboard { u: [1 << 51,       0] },
        Bitboard { u: [1 << 52,       0] },
        Bitboard { u: [1 << 53,       0] },
        Bitboard { u: [1 << 54,       0] },
        Bitboard { u: [1 << 55,       0] },
        Bitboard { u: [1 << 56,       0] },
        Bitboard { u: [1 << 57,       0] },
        Bitboard { u: [1 << 58,       0] },
        Bitboard { u: [1 << 59,       0] },
        Bitboard { u: [1 << 60,       0] },
        Bitboard { u: [1 << 61,       0] },
        Bitboard { u: [1 << 62,       0] },
        Bitboard { u: [      0, 1 <<  0] },
        Bitboard { u: [      0, 1 <<  1] },
        Bitboard { u: [      0, 1 <<  2] },
        Bitboard { u: [      0, 1 <<  3] },
        Bitboard { u: [      0, 1 <<  4] },
        Bitboard { u: [      0, 1 <<  5] },
        Bitboard { u: [      0, 1 <<  6] },
        Bitboard { u: [      0, 1 <<  7] },
        Bitboard { u: [      0, 1 <<  8] },
        Bitboard { u: [      0, 1 <<  9] },
        Bitboard { u: [      0, 1 << 10] },
        Bitboard { u: [      0, 1 << 11] },
        Bitboard { u: [      0, 1 << 12] },
        Bitboard { u: [      0, 1 << 13] },
        Bitboard { u: [      0, 1 << 14] },
        Bitboard { u: [      0, 1 << 15] },
        Bitboard { u: [      0, 1 << 16] },
        Bitboard { u: [      0, 1 << 17] },
    ];
}

impl ops::Not for Bitboard {
    type Output = Bitboard;

    fn not(self) -> Self::Output {
        Bitboard {
            u: [!self.value(0), !self.value(1)],
        }
    }
}

impl ops::BitAnd<Bitboard> for Bitboard {
    type Output = Bitboard;

    fn bitand(self, rhs: Bitboard) -> Self::Output {
        Bitboard {
            u: [self.value(0) & rhs.value(0), self.value(1) & rhs.value(1)],
        }
    }
}

impl ops::BitAnd<Square> for Bitboard {
    type Output = Bitboard;

    fn bitand(self, rhs: Square) -> Self::Output {
        self & Bitboard::from_square(rhs)
    }
}

impl ops::BitOr<Bitboard> for Bitboard {
    type Output = Bitboard;

    fn bitor(self, rhs: Bitboard) -> Self::Output {
        Bitboard {
            u: [self.value(0) | rhs.value(0), self.value(1) | rhs.value(1)],
        }
    }
}

impl ops::BitOr<Square> for Bitboard {
    type Output = Bitboard;

    fn bitor(self, rhs: Square) -> Self::Output {
        self | Bitboard::from_square(rhs)
    }
}

impl ops::BitAndAssign<Bitboard> for Bitboard {
    fn bitand_assign(&mut self, rhs: Bitboard) {
        unsafe {
            self.u[0] &= rhs.u[0];
            self.u[1] &= rhs.u[1];
        }
    }
}

impl ops::BitAndAssign<Square> for Bitboard {
    fn bitand_assign(&mut self, rhs: Square) {
        *self &= Bitboard::from_square(rhs);
    }
}

impl ops::BitOrAssign<Bitboard> for Bitboard {
    fn bitor_assign(&mut self, rhs: Bitboard) {
        unsafe {
            self.u[0] |= rhs.u[0];
            self.u[1] |= rhs.u[1];
        }
    }
}

impl ops::BitOrAssign<Square> for Bitboard {
    fn bitor_assign(&mut self, rhs: Square) {
        *self |= Bitboard::from_square(rhs)
    }
}

impl ops::BitXorAssign<Bitboard> for Bitboard {
    fn bitxor_assign(&mut self, rhs: Bitboard) {
        unsafe {
            self.u[0] ^= rhs.u[0];
            self.u[1] ^= rhs.u[1];
        }
    }
}

impl ops::BitXorAssign<Square> for Bitboard {
    fn bitxor_assign(&mut self, rhs: Square) {
        *self ^= Bitboard::from_square(rhs)
    }
}

impl Iterator for Bitboard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        self.pop()
    }
}

impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        for &rank in Rank::ALL.iter() {
            s.push(' ');
            for &file in File::ALL.iter().rev() {
                let b = !(*self & Square::new(file, rank)).is_empty();
                s.push(if b { '#' } else { '.' });
            }
            s += "\n";
        }
        write!(f, "Bitboard(\n{})", s)
    }
}
