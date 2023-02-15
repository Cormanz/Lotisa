use crate::{boards::{perft, Board}, communication::UCICommunicator};

#[ignore]
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
    //assert_eq!(perft(&mut uci, 6, 0, None), 119060324, "Perft Test (depth = 6)");
}


#[test]
fn a4_b5_Nf3_perft() {
    
    let mut uci = UCICommunicator {
        board: Board::load_fen("rnbqkbnr/p1pppppp/8/1P6/8/8/1PPPPPPP/RNBQKBNR")
    };
    uci.board.print_board();

    // Perft Results sourced on WebPerft (https://analog-hors.github.io/webperft/)

    assert_eq!(perft(&mut uci, 2, 0, None), 502, "Perft Test (depth = 3)");
}