use std::ops;

/// Represent a color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Black = 0,
    White = 1,
}

impl Color {
    pub const NUM: usize = 2;
    pub const ALL: [Color; Color::NUM] = [Color::Black, Color::White];

    pub fn index(&self) -> usize {
        *self as usize
    }
}

impl ops::Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}
