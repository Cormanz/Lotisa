use crate::{
    boards::{perft, Board},
    communication::{UCICommunicator, Communicator},
};

#[test]
fn startpos_perft() {
    let mut uci = Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - -");

    // Perft Results sourced on WebPerft (https://analog-hors.github.io/webperft/)

    assert_eq!(perft(&mut uci, 1, None), 20, "Perft Test (depth = 1)");
    assert_eq!(perft(&mut uci, 2, None), 400, "Perft Test (depth = 2)");
    assert_eq!(perft(&mut uci, 3, None), 8902, "Perft Test (depth = 3)");
    assert_eq!(
        perft(&mut uci, 4, None),
        197281,
        "Perft Test (depth = 4)"
    );
    assert_eq!(
        perft(&mut uci, 5, None),
        4865609,
        "Perft Test (depth = 5)"
    );
}

#[test]
fn en_passant_perft() {
    let mut uci = Board::load_fen("rnbqkbnr/p1pppppp/8/1P6/8/8/1PPPPPPP/RNBQKBNR b - -");

    // Perft Results sourced on WebPerft (https://analog-hors.github.io/webperft/)

    assert_eq!(perft(&mut uci, 3, None), 11204, "Perft Test (depth = 3)");
}

#[test]
fn castling_test() {
    let mut uci = Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/R3KBNR w kqKQ -");

    // Perft Results sourced on WebPerft (https://analog-hors.github.io/webperft/)

    assert_eq!(
        perft(&mut uci, 4, None),
        236936,
        "Perft Test (depth = 4)"
    );
}

#[test]
fn promotion_test() {
    let mut uci = Board::load_fen("8/5P2/8/8/8/7K/8/n6k w - -");

    // Perft Results sourced on WebPerft (https://analog-hors.github.io/webperft/)

    assert_eq!(perft(&mut uci, 3, None), 299, "Perft Test (depth = 3)");
}

#[test]
fn team_switch() {
    let uci = Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -");

    assert_eq!(uci.board.moving_team, 0);
    assert_eq!(uci.board.next_team(), 1);
    assert_eq!(uci.board.previous_team(), 1);

}