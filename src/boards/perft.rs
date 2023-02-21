use crate::communication::{Communicator, UCICommunicator};

use super::{Action, Board};

pub fn perft_psuedolegal(
    uci: &mut UCICommunicator,
    depth: i16,
    last_action: Option<Action>,
) -> u64 {
    let mut nodes: u64 = 0;

    let actions = uci.board.generate_legal_moves();
    if depth == 1 {
        return actions.len() as u64;
    }

    for action in &actions {
        uci.board.make_move(*action);
        nodes += perft(uci, depth - 1, Some(*action));
        uci.board.undo_move();
    }

    nodes
}

pub fn perft(uci: &mut UCICommunicator, depth: i16, last_action: Option<Action>) -> u64 {
    let mut nodes: u64 = 0;

    let actions = uci.board.generate_legal_moves();
    if depth == 1 {
        return actions.len() as u64;
    }

    for action in &actions {
        uci.board.make_move(*action);
        nodes += perft(uci, depth - 1, Some(*action));
        uci.board.undo_move();
    }

    nodes
}
