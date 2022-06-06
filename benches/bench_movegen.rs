#![feature(test)]
extern crate test;

#[cfg(test)]
mod movegen {
    use shogi_core::{Color, Piece};
    use test::Bencher;
    use yasai::Position;

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
                None, Some(Piece::W_K), None, None, None, None, None, None, None,
                None, Some(Piece::B_S), None, None, None, None, None, None, None,
                None, Some(Piece::B_S), None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, Some(Piece::B_L),
                None, Some(Piece::B_S), Some(Piece::B_B), None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, Some(Piece::B_L),
                None, Some(Piece::B_K), None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, Some(Piece::B_L),
                Some(Piece::B_R), None, None, None, None, None, None, None, None,
            ], [
                [ 1, 1, 1, 1, 1, 1, 1, 0],
                [17, 0, 3, 0, 3, 0, 0, 0],
            ], Color::Black, 1);
            assert_eq!(593, pos.legal_moves().len());
        });
    }
}
