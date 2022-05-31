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
                            None, Some(Piece::WOU),             None,             None,             None,             None,             None,             None,             None,
                            None, Some(Piece::BGI),             None,             None,             None,             None,             None,             None,             None,
                            None, Some(Piece::BGI),             None,             None,             None,             None,             None,             None,             None,
                            None,             None,             None,             None,             None,             None,             None,             None, Some(Piece::BKY),
                            None, Some(Piece::BGI), Some(Piece::BKA),             None,             None,             None,             None,             None,             None,
                            None,             None,             None,             None,             None,             None,             None,             None, Some(Piece::BKY),
                            None, Some(Piece::BOU),             None,             None,             None,             None,             None,             None,             None,
                            None,             None,             None,             None,             None,             None,             None,             None, Some(Piece::BKY),
                Some(Piece::BHI),             None,             None,             None,             None,             None,             None,             None,             None,
            ], [
                [ 1, 1, 1, 1, 1, 1, 1],
                [17, 0, 3, 0, 3, 0, 0],
            ], Color::Black, 1);
            assert_eq!(593, pos.legal_moves().len());
        });
    }
}
