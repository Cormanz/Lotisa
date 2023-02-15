use crate::communication::{UCICommunicator, Communicator};

use super::{Board, Action};

pub fn perft(uci: &mut UCICommunicator, depth: i16, team: i16, last_action: Option<Action>) -> u64 {
    let mut nodes: u64 = 0;

    let actions = uci.board.generate_legal_moves(team);
    if depth == 1 {
        uci.board.undo_move();
        println!("{}", uci.encode(&last_action.unwrap()));
        uci.board.make_move(last_action.unwrap());
        for action in &actions {
            //println!("AHAHAH {}", uci.encode(action));
        }
        return actions.len() as u64;
    }

    for action in &actions {
        uci.board.make_move(*action);
        nodes += perft(uci, depth - 1, if team == 0 { 1 } else { 0 }, Some(*action));
        uci.board.undo_move();
    }

    if depth == 2 {
        println!("HO HO {}", &actions.len());
    }

    if depth == 112 {
        uci.board.undo_move();
        println!("AHAHAH {} {}", nodes, uci.encode(&last_action.unwrap()));
        uci.board.make_move(last_action.unwrap());
    }

    nodes
}
