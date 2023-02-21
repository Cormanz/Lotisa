use crate::boards::{perft, Board};

#[test]
fn startpos_perft() {
    let mut uci = Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - -");

    // Perft Results sourced on WebPerft (https://analog-hors.github.io/webperft/)

    assert_eq!(perft(&mut uci, 1, None), 20, "Perft Test (depth = 1)");
    assert_eq!(perft(&mut uci, 2, None), 400, "Perft Test (depth = 2)");
    assert_eq!(perft(&mut uci, 3, None), 8902, "Perft Test (depth = 3)");
    assert_eq!(perft(&mut uci, 4, None), 197281, "Perft Test (depth = 4)");
    assert_eq!(perft(&mut uci, 5, None), 4865609, "Perft Test (depth = 5)");
}

#[test]
fn en_passant_perft() {
    let mut uci = Board::load_fen("rnbqkbnr/p1pppppp/8/1P6/8/8/1PPPPPPP/RNBQKBNR b - -");

    // Perft Results sourced on WebPerft (https://analog-hors.github.io/webperft/)

    assert_eq!(perft(&mut uci, 3, None), 11204, "Perft Test (depth = 3)");
}

#[test]
fn en_passant_uci_test() {
    /*
        This is meant to address a bug where my UCI implementation incorrectly believes the previous move was a double move and allows for en passant.
    */

    let mut uci = Board::load_uci_pgn("1. d2d4 d7d5 2. c2c4 c7c6 3. c1c3 g8f6 4. e2e3 g7g6 5. f1e2 b8d7 6. b2b4 f6e4 7. c3e4 d5e4");
    assert!(
        !uci.board
            .generate_legal_moves()
            .iter()
            .any(|action| action.piece_type == 0 && action.info == -3),
        "There is an en passant in this position when there shouldn't be."
    );
}

#[test]
fn castling_test() {
    let mut uci = Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/R3KBNR w kqKQ -");

    // Perft Results sourced on WebPerft (https://analog-hors.github.io/webperft/)

    assert_eq!(perft(&mut uci, 4, None), 236936, "Perft Test (depth = 4)");
}

#[test]
fn white_promotion_test() {
    let mut uci = Board::load_fen("8/5P2/8/8/8/7K/8/n6k w - -");

    // Perft Results sourced on WebPerft (https://analog-hors.github.io/webperft/)

    assert_eq!(perft(&mut uci, 3, None), 299, "Perft Test (depth = 3)");
}

#[test]
fn black_promotion_test() {
    /*
        This is meant to address a bug where my UCI implementation incorrectly believes the previous move was a double move and allows for en passant.
    */

    let mut uci = Board::load_fen("rnb1kb1r/pppn2pp/4p3/8/8/2N1KPP1/PPP1p2P/R4BNR b kq -");
    uci.board.print_board();

    assert!(
        uci.board
            .generate_legal_moves()
            .iter()
            .any(|action| action.piece_type != 0 || action.info >= 0),
        "There aren't any promotions in this position when there should be."
    );
}

#[test]
fn team_switch() {
    let uci = Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -");

    assert_eq!(uci.board.moving_team, 0);
    assert_eq!(uci.board.next_team(), 1);
    assert_eq!(uci.board.previous_team(), 1);
}
