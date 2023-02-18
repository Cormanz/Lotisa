use std::io::{Stdin, BufRead};
use rand::{SeedableRng, seq::SliceRandom};

use crate::{boards::Board, engine::{SearchInfo, search, MIN_VALUE, MAX_VALUE, root_search}, communication::Communicator};

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
                let action = uci.decode(action.to_string());
                uci.board.make_move(action);
            }
        } else if line.starts_with("go") {
            let mut info = SearchInfo {
                root_depth: 0,
                best_move: None,
                time: 0
            };
            let moving_team = uci.board.moving_team;
            let score = root_search(&mut info, &mut uci.board, moving_team, 50);
            let best_move = &info.best_move.unwrap();
            let mut rng = rand_hc::Hc128Rng::from_entropy();
            println!("info depth {} time {} cp {}", info.root_depth, info.time, score / 10);
            println!("bestmove {}", uci.encode(&best_move));
        } else if line == "isready" {
            println!("readyok");
        }
    }
}