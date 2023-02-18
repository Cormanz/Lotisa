use std::io::{Stdin, BufRead};
use crate::{boards::Board, engine::{SearchInfo, search, MIN_VALUE, MAX_VALUE}, communication::Communicator};

pub fn run_uci(stdin: Stdin) {
    let mut uci = Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w kqKQ -");

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        if line == "ucinewgame" {
            uci = Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w kqKQ -");
        } else if line.starts_with("position startpos moves ") {
            let moves = &line[24..].split(" ").collect::<Vec<_>>();
            uci = Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w kqKQ -");
            for action in moves {
                uci.board.make_move(uci.decode(action.to_string()));
            }
        } else if line.starts_with("go") {
            let mut info = SearchInfo {
                root_depth: 5,
                best_move: None
            };
            let moving_team = uci.board.moving_team;
            search(&mut info, &mut uci.board, MIN_VALUE, MAX_VALUE, 5, 0, moving_team);
            println!("bestmove {}", uci.encode(&info.best_move.unwrap()));
        } else if line == "isready" {
            println!("readyok");
        }
    }
}