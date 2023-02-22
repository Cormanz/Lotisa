use crate::{boards::Board, communication::Communicator};

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

#[test]
fn capture_test() {
    let mut uci = Board::load_fen("8/1k6/8/8/4p3/5P2/1K6/8 b - -");

    println!("{}", uci.decode_pos("a4".to_string()));
}
