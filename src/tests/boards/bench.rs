use std::fmt::Debug;

use crate::{
    boards::{perft, perft_psuedolegal, Board},
    engine::get_epoch_ms,
};

pub fn bench<T: Debug>(benchmark: &str, run: &dyn Fn() -> T) {
    let start = get_epoch_ms();
    let result = run();
    let end = get_epoch_ms();
    println!("{} | result: {:?} time: {}", benchmark, result, end - start);
}

pub fn psuedolegal_moves() {
    bench("perft psuedolegal", &|| {
        let mut uci = Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - -");
        perft_psuedolegal(&mut uci, 5, None)
    });
}

pub fn legal_moves() {
    bench("perft legal", &|| {
        let mut uci = Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - -");
        perft(&mut uci, 5, None)
    });
}

pub fn boards_bench() {
    psuedolegal_moves();
    legal_moves();
}
