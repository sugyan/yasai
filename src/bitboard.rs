use crate::Square;
use std::{fmt, ops};

#[derive(Clone, Copy)]
pub union Bitboard {
    u: [u64; 2],
}

impl Bitboard {
    pub const ZERO: Bitboard = Bitboard { u: [0, 0] };

    pub fn is_empty(&self) -> bool {
        (self.value(0) | self.value(1)) == 0
    }
    pub fn pop(&mut self) -> Square {
        if self.value(0) != 0 {
            self.pop0()
        } else {
            self.pop1()
        }
    }
    fn value(&self, i: usize) -> u64 {
        debug_assert!(i < 2);
        unsafe { *self.u.get_unchecked(i) }
    }
    fn pop0(&mut self) -> Square {
        let sq = Square(self.value(0).trailing_zeros() as i8);
        unsafe {
            self.u[0] &= self.u[0] - 1;
        }
        sq
    }
    fn pop1(&mut self) -> Square {
        let sq = Square(self.value(1).trailing_zeros() as i8);
        unsafe {
            self.u[1] &= self.u[1] - 1;
        }
        sq
    }

    const SQUARE_MASK: [Bitboard; Square::NUM] = [
        Bitboard { u: [1 << 0, 0] },
        Bitboard { u: [1 << 1, 0] },
        Bitboard { u: [1 << 2, 0] },
        Bitboard { u: [1 << 3, 0] },
        Bitboard { u: [1 << 4, 0] },
        Bitboard { u: [1 << 5, 0] },
        Bitboard { u: [1 << 6, 0] },
        Bitboard { u: [1 << 7, 0] },
        Bitboard { u: [1 << 8, 0] },
        Bitboard { u: [1 << 9, 0] },
        Bitboard { u: [1 << 10, 0] },
        Bitboard { u: [1 << 11, 0] },
        Bitboard { u: [1 << 12, 0] },
        Bitboard { u: [1 << 13, 0] },
        Bitboard { u: [1 << 14, 0] },
        Bitboard { u: [1 << 15, 0] },
        Bitboard { u: [1 << 16, 0] },
        Bitboard { u: [1 << 17, 0] },
        Bitboard { u: [1 << 18, 0] },
        Bitboard { u: [1 << 19, 0] },
        Bitboard { u: [1 << 20, 0] },
        Bitboard { u: [1 << 21, 0] },
        Bitboard { u: [1 << 22, 0] },
        Bitboard { u: [1 << 23, 0] },
        Bitboard { u: [1 << 24, 0] },
        Bitboard { u: [1 << 25, 0] },
        Bitboard { u: [1 << 26, 0] },
        Bitboard { u: [1 << 27, 0] },
        Bitboard { u: [1 << 28, 0] },
        Bitboard { u: [1 << 29, 0] },
        Bitboard { u: [1 << 30, 0] },
        Bitboard { u: [1 << 31, 0] },
        Bitboard { u: [1 << 32, 0] },
        Bitboard { u: [1 << 33, 0] },
        Bitboard { u: [1 << 34, 0] },
        Bitboard { u: [1 << 35, 0] },
        Bitboard { u: [1 << 36, 0] },
        Bitboard { u: [1 << 37, 0] },
        Bitboard { u: [1 << 38, 0] },
        Bitboard { u: [1 << 39, 0] },
        Bitboard { u: [1 << 40, 0] },
        Bitboard { u: [1 << 41, 0] },
        Bitboard { u: [1 << 42, 0] },
        Bitboard { u: [1 << 43, 0] },
        Bitboard { u: [1 << 44, 0] },
        Bitboard { u: [1 << 45, 0] },
        Bitboard { u: [1 << 46, 0] },
        Bitboard { u: [1 << 47, 0] },
        Bitboard { u: [1 << 48, 0] },
        Bitboard { u: [1 << 49, 0] },
        Bitboard { u: [1 << 50, 0] },
        Bitboard { u: [1 << 51, 0] },
        Bitboard { u: [1 << 52, 0] },
        Bitboard { u: [1 << 53, 0] },
        Bitboard { u: [1 << 54, 0] },
        Bitboard { u: [1 << 55, 0] },
        Bitboard { u: [1 << 56, 0] },
        Bitboard { u: [1 << 57, 0] },
        Bitboard { u: [1 << 58, 0] },
        Bitboard { u: [1 << 59, 0] },
        Bitboard { u: [1 << 60, 0] },
        Bitboard { u: [1 << 61, 0] },
        Bitboard { u: [1 << 62, 0] },
        Bitboard { u: [0, 1 << 0] },
        Bitboard { u: [0, 1 << 1] },
        Bitboard { u: [0, 1 << 2] },
        Bitboard { u: [0, 1 << 3] },
        Bitboard { u: [0, 1 << 4] },
        Bitboard { u: [0, 1 << 5] },
        Bitboard { u: [0, 1 << 6] },
        Bitboard { u: [0, 1 << 7] },
        Bitboard { u: [0, 1 << 8] },
        Bitboard { u: [0, 1 << 9] },
        Bitboard { u: [0, 1 << 10] },
        Bitboard { u: [0, 1 << 11] },
        Bitboard { u: [0, 1 << 12] },
        Bitboard { u: [0, 1 << 13] },
        Bitboard { u: [0, 1 << 14] },
        Bitboard { u: [0, 1 << 15] },
        Bitboard { u: [0, 1 << 16] },
        Bitboard { u: [0, 1 << 17] },
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
        self & Bitboard::SQUARE_MASK[rhs.0 as usize]
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
        *self |= Bitboard::SQUARE_MASK[rhs.0 as usize];
    }
}

impl Iterator for Bitboard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            None
        } else {
            Some(self.pop())
        }
    }
}

impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        for rank in 0..9 {
            s.push(' ');
            for file in (0..9).rev() {
                s.push(if (*self & Square::new(file, rank)).is_empty() {
                    '.'
                } else {
                    '#'
                });
            }
            s += "\n";
        }
        write!(f, "Bitboard(\n{})", s)
    }
}
