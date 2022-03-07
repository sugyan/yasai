#![feature(test)]
extern crate test;

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;
    use yasai::{Color, Piece, Position};

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
            assert_eq!(53_393_368, perft(&mut pos, 3));
        });
    }
}
