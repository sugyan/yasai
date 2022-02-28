mod bitboard;
mod color;
mod movegen;
mod piece;
mod position;
mod shogi_move;
mod square;
mod attack_table;

pub use color::Color;
pub use piece::{Piece, PieceType};
pub use position::Position;
pub use shogi_move::Move;
pub use square::Square;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
