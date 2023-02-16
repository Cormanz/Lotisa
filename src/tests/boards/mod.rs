use crate::{boards::{perft, Board}, communication::UCICommunicator};

#[test]
fn startpos_perft() {
    let mut uci = UCICommunicator {
        board: Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR")
    };

    // Perft Results sourced on WebPerft (https://analog-hors.github.io/webperft/)

    assert_eq!(perft(&mut uci, 1, 0, None), 20, "Perft Test (depth = 1)");
    assert_eq!(perft(&mut uci, 2, 0, None), 400, "Perft Test (depth = 2)");
    assert_eq!(perft(&mut uci, 3, 0, None), 8902, "Perft Test (depth = 3)");
    assert_eq!(perft(&mut uci, 4, 0, None), 197281, "Perft Test (depth = 4)");
    assert_eq!(perft(&mut uci, 5, 0, None), 4865609, "Perft Test (depth = 5)");
}


#[test]
fn en_passant_a4_b5_xb5_perft() {
    let mut uci = UCICommunicator {
        board: Board::load_fen("rnbqkbnr/p1pppppp/8/1P6/8/8/1PPPPPPP/RNBQKBNR")
    };

    // Perft Results sourced on WebPerft (https://analog-hors.github.io/webperft/)

    assert_eq!(perft(&mut uci, 3, 1, None), 11204, "Perft Test (depth = 3)");
}

#[test]
fn castling_test() {
    let mut uci = UCICommunicator {
        board: Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/R3KBNR")
    };

    // Perft Results sourced on WebPerft (https://analog-hors.github.io/webperft/)

    assert_eq!(perft(&mut uci, 3, 1, None), 10237, "Perft Test (depth = 3)");
    assert_eq!(perft(&mut uci, 4, 1, None), 236407, "Perft Test (depth = 4)");

}