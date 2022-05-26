use shogi_core::{Color, PieceKind};

pub(crate) trait ArrayIndex {
    fn array_index(self) -> usize;
}

impl ArrayIndex for Color {
    fn array_index(self) -> usize {
        self as usize - 1
    }
}

impl ArrayIndex for PieceKind {
    fn array_index(self) -> usize {
        self as usize - 1
    }
}
