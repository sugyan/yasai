use shogi_core::ToUsi;
use std::process;
use std::time::Instant;
use yasai::Position;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if let Some(depth) = args.get(1).and_then(|s| s.parse().ok()) {
        let mut pos = Position::default();
        assert_eq!(30, pos.legal_moves().len());
        let now = Instant::now();
        let total = perft(&mut pos, depth, true);
        let duration = now.elapsed();
        println!();
        println!("Time duration: {:?}", duration);
        println!(
            "Searched: {total} nodes: {} nps",
            (total as u128) * 1_000_000_000 / duration.as_nanos()
        );
    } else {
        println!("usage: perft <depth>");
        process::exit(1);
    }
}

fn perft(pos: &mut Position, depth: usize, is_root: bool) -> usize {
    let mut ret = 0;
    for m in pos.legal_moves() {
        let count = if depth <= 1 {
            1
        } else {
            pos.do_move(m);
            let ret = if depth == 2 {
                pos.legal_moves().len()
            } else {
                perft(pos, depth - 1, false)
            };
            pos.undo_move(m);
            ret
        };
        ret += count;
        if is_root {
            println!("{}: {count}", m.to_usi_owned());
        }
    }
    ret
}
