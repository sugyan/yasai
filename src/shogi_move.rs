use crate::{Piece, Square};

#[derive(Clone, Copy, Debug)]
pub struct Move {
    from: Option<Square>,
    to: Square,
    promotion: bool,
    piece: Piece,
}

impl Move {
    fn new(from: Option<Square>, to: Square, promotion: bool, piece: Piece) -> Self {
        Move {
            from,
            to,
            promotion,
            piece,
        }
    }
    pub fn new_normal(from: Square, to: Square, promotion: bool, piece: Piece) -> Self {
        Move::new(Some(from), to, promotion, piece)
    }
    pub fn new_drop(to: Square, piece: Piece) -> Self {
        Move::new(None, to, false, piece)
    }
    pub fn from(&self) -> Option<Square> {
        self.from
    }
    pub fn to(&self) -> Square {
        self.to
    }
    pub fn is_promotion(&self) -> bool {
        self.promotion
    }
    pub fn piece(&self) -> Piece {
        self.piece
    }
}
