#![feature(test)]
extern crate test;

#[cfg(test)]
mod movegen {
    use shogi_core::PartialPosition;
    use shogi_usi_parser::FromUsi;
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
            let pos = Position::new(
                PartialPosition::from_usi(
                    "sfen R8/2K1S1SSk/4B4/9/9/9/9/9/1L1L1L3 b RBGSNLP3g3n17p 1",
                )
                .expect("failed to parse"),
            );
            assert_eq!(593, pos.legal_moves().len());
        });
    }
}
