use std::io::{Stdin, BufRead};
use rand::{SeedableRng, seq::SliceRandom};
use regex::Regex;

use crate::{boards::Board, engine::{SearchInfo, search, MIN_VALUE, MAX_VALUE, root_search, PV, MAX_DEPTH, MAX_KILLER_MOVES}, communication::Communicator};

pub fn create_info() -> SearchInfo {
    SearchInfo {
        root_depth: 0,
        root_nodes: 0,
        quiescence_nodes: 0,
        time: 0,
        pv_table: PV { table: [ [ None; MAX_DEPTH ]; MAX_DEPTH ], length: [ 0; MAX_DEPTH ] },
        transposition_table: vec![None; 9_000_000],
        max_tt_size: 9_000_000,
        killer_moves: [[None; MAX_DEPTH]; MAX_KILLER_MOVES],
        history_moves: vec![vec![vec![0; 120]; 120]; 2]
    }
}

pub fn run_uci(stdin: Stdin) {
    let mut uci = Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w kqKQ -");
    let mut info = create_info();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        if line == "ucinewgame" {
            uci = Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w kqKQ -");
            info = create_info();
        } else if line.starts_with("position startpos moves ") {
            let moves = &line[24..].split(" ").collect::<Vec<_>>();
            uci = Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w kqKQ -");
            for action in moves {
                let action = uci.decode(action.to_string());
                uci.board.make_move(action);
            }
        } else if line.starts_with("position startpos fen ") {
            let fen = &line[22..];
            uci = Board::load_fen(fen);
            info = create_info();
        } else if line.starts_with("print-board") {
            // Not UCI but why not

            uci.board.print_board();
        } else if line.starts_with("go") {
            let mut max_time = 0;

            let wtime_re = Regex::new(r"wtime (\d+)").unwrap();
            let btime_re = Regex::new(r"btime (\d+)").unwrap();
            let wtime_inc = Regex::new(r"winc (\d+)").unwrap();
            let btime_inc = Regex::new(r"binc (\d+)").unwrap();
            
            let (time_re, inc_re) = match uci.board.moving_team {
                0 => (wtime_re, wtime_inc),
                1 => (btime_re, btime_inc),
                _ => (wtime_re, wtime_inc)
            };

            let mut found_capture = false;

            if let Some(cap) = time_re.captures(&line) {
                max_time = cap[1].parse::<u128>().unwrap() / 300;
                found_capture = true;
            }

            if let Some(cap) = inc_re.captures(&line) {
                max_time += cap[1].parse::<u128>().unwrap() / 10;
                found_capture = true;
            }

            if !found_capture {
                max_time = 50;
            }

            let moving_team = uci.board.moving_team;
            let score = root_search(&mut info, &mut uci, moving_team, max_time);
            let best_move = info.pv_table.table[0][0];
            if let Some(best_move) = best_move {
                println!("bestmove {}", uci.encode(&best_move));
            } else {}
        } else if line == "isready" {
            println!("readyok");
        }
    }
}