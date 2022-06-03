#![feature(test)]
extern crate test;

#[cfg(test)]
mod perft {
    use shogi_core::{Color, Piece};
    use test::Bencher;
    use yasai::Position;

    fn perft(pos: &mut Position, depth: usize) -> usize {
        let mut ret = 0;
        for m in pos.legal_moves() {
            let count = if depth <= 1 {
                1
            } else {
                pos.do_move(m);
                let ret = if depth == 2 {
                    pos.legal_moves().len()
                } else {
                    perft(pos, depth - 1)
                };
                pos.undo_move(m);
                ret
            };
            ret += count;
        }
        ret
    }

    #[bench]
    fn bench_perft_from_default(b: &mut Bencher) {
        b.iter(|| {
            let mut pos = Position::default();
            assert_eq!(19_861_490, perft(&mut pos, 5));
        });
    }

    #[bench]
    fn bench_perft_from_maximum_moves(b: &mut Bencher) {
        b.iter(|| {
            #[rustfmt::skip]
            let mut pos = Position::new([
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
            assert_eq!(53_393_368, perft(&mut pos, 3));
        });
    }
}
