use shogi_core::Color;

pub(crate) trait Index {
    fn index(&self) -> usize;
}

impl Index for Color {
    fn index(&self) -> usize {
        match self {
            Color::Black => 0,
            Color::White => 1,
        }
    }
}
