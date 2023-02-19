use std::io::{Stdin, BufRead};
use rand::{SeedableRng, seq::SliceRandom};

use crate::{boards::Board, engine::{SearchInfo, search, MIN_VALUE, MAX_VALUE, root_search, PV}, communication::Communicator};

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
            let pv_table = PV { table: [ [ None; 100 ]; 100 ], length: [ 0; 100 ] };
            let mut info = SearchInfo {
                root_depth: 0,
                search_nodes: 0,
                best_move: None,
                time: 0,
                pv_table
            };
            let moving_team = uci.board.moving_team;
            let score = root_search(&mut info, &mut uci.board, moving_team, 1000);
            let best_move = &info.best_move;
            //let mut rng = rand_hc::Hc128Rng::from_entropy();
            println!(
                "info depth {} time {} cp {} pv {} nodes {} nps {}", 
                info.root_depth, info.time, score / 10, info.pv_table.display_pv(&mut uci), info.search_nodes, (info.search_nodes / info.time) * 1000
            );
            if let Some(best_move) = best_move {
                println!("bestmove {}", uci.encode(&best_move));
            } else {}
        } else if line == "isready" {
            println!("readyok");
        }
    }
}