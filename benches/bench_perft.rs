#![feature(test)]
extern crate test;

#[cfg(test)]
mod perft {
    use shogi_core::PartialPosition;
    use shogi_usi_parser::FromUsi;
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
    fn bench_perft_5_from_default(b: &mut Bencher) {
        b.iter(|| {
            let mut pos = Position::default();
            assert_eq!(19_861_490, perft(&mut pos, 5));
        });
    }

    #[bench]
    fn bench_perft_3_from_maximum_moves(b: &mut Bencher) {
        b.iter(|| {
            let mut pos = Position::new(
                PartialPosition::from_usi(
                    "sfen R8/2K1S1SSk/4B4/9/9/9/9/9/1L1L1L3 b RBGSNLP3g3n17p 1",
                )
                .expect("failed to parse"),
            );
            assert_eq!(53_393_368, perft(&mut pos, 3));
        });
    }
}
