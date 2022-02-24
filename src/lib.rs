mod piece;
mod position;
mod square;

pub use piece::Piece;
pub use position::Position;
pub use square::Square;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
