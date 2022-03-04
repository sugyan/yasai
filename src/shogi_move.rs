use crate::{Piece, PieceType, Square};

#[derive(Clone, Copy, Debug)]
pub struct Move {
    from: Option<Square>,
    to: Square,
    promotion: bool,
    piece_type: PieceType,
}

impl Move {
    fn new(from: Option<Square>, to: Square, promotion: bool, piece_type: PieceType) -> Self {
        Move {
            from,
            to,
            promotion,
            piece_type,
        }
    }
    pub fn new_normal(from: Square, to: Square, promotion: bool, piece: Piece) -> Self {
        Move::new(
            Some(from),
            to,
            promotion,
            piece
                .piece_type()
                .expect("failed to crete move for invalid piece"),
        )
    }
    pub fn new_drop(to: Square, piece_type: PieceType) -> Self {
        Move::new(None, to, false, piece_type)
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
    pub fn is_drop(&self) -> bool {
        self.from.is_none()
    }
    pub fn piece_type(&self) -> PieceType {
        self.piece_type
    }
}
