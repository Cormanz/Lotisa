use crate::boards::{Board, perft};

#[test]
fn startpos_perft() {
    let mut board = Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
    
    // Perft Results sourced on WebPerft (https://analog-hors.github.io/webperft/)

    assert_eq!(perft(&mut board, 1, 0), 20, "Perft Test (depth = 1)");
    assert_eq!(perft(&mut board, 2, 0), 400, "Perft Test (depth = 2)");
    assert_eq!(perft(&mut board, 3, 0), 8902, "Perft Test (depth = 3)");
    assert_eq!(perft(&mut board, 4, 0), 197281, "Perft Test (depth = 4)");
    assert_eq!(perft(&mut board, 5, 0), 4865609, "Perft Test (depth = 5)");
}