use crate::{Piece, Square};

#[derive(Clone, Copy, Debug)]
pub struct Move {
    from: Option<Square>,
    to: Square,
    promotion: bool,
    piece: Piece,
    captured: Piece,
}

impl Move {
    fn new(
        from: Option<Square>,
        to: Square,
        promotion: bool,
        piece: Piece,
        captured: Piece,
    ) -> Self {
        Move {
            from,
            to,
            promotion,
            piece,
            captured,
        }
    }
    pub fn new_normal(
        from: Square,
        to: Square,
        promotion: bool,
        piece: Piece,
        captured: Piece,
    ) -> Self {
        Move::new(Some(from), to, promotion, piece, captured)
    }
    pub fn new_drop(to: Square, piece: Piece) -> Self {
        Move::new(None, to, false, piece, Piece::EMP)
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
    pub fn captured(&self) -> Option<Piece> {
        if self.captured != Piece::EMP {
            Some(self.captured)
        } else {
            None
        }
    }
}
