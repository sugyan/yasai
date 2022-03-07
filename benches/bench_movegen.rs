#![feature(test)]
extern crate test;

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;
    use yasai::{Color, Piece, Position};

    #[bench]
    fn bench_legal_moves_from_default(b: &mut Bencher) {
        b.iter(|| {
            let pos = Position::default();
            assert_eq!(30, pos.legal_moves().len());
        });
    }

    #[bench]
    fn bench_legal_moves_maximum(b: &mut Bencher) {
        b.iter(|| {
            #[rustfmt::skip]
            let pos = Position::new([
                Piece::EMP, Piece::WOU, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP,
                Piece::EMP, Piece::BGI, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP,
                Piece::EMP, Piece::BGI, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP,
                Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::BKY,
                Piece::EMP, Piece::BGI, Piece::BKA, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP,
                Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::BKY,
                Piece::EMP, Piece::BOU, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP,
                Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::BKY,
                Piece::BHI, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP,
            ], [
                [ 1, 1, 1, 1, 1, 1, 1],
                [17, 0, 3, 0, 3, 0, 0],
            ], Color::Black);
            assert_eq!(593, pos.legal_moves().len());
        });
    }
}
