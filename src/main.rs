use core::time;
use communication::UCICommunicator;
use engine::{eval_board, negamax_root};
use rand::seq::{IteratorRandom, SliceRandom};
use std::{
    env, thread,
    time::{Duration, SystemTime, UNIX_EPOCH}, io::{self, BufRead},
};

use boards::Board;

use crate::{engine::{create_search_info, negamax_deepening}, boards::in_check, communication::Communicator};

mod boards;
mod engine;
mod communication;

fn get_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub fn perft_psuedo(board: &mut Board, depth: i16, team: i16) -> u64 {
    let mut nodes: u64 = 0;

    let actions = board.generate_moves(team);
    if depth == 1 {
        return actions.len() as u64;
    }

    for action in actions {
        let undo = board.make_move(action);
        nodes += perft_psuedo(board, depth - 1, if team == 0 { 1 } else { 0 });
        board.undo_move(undo);
    }

    nodes
}

pub fn perft(board: &mut Board, depth: i16, team: i16) -> u64 {
    let mut nodes: u64 = 0;

    let actions = board.generate_legal_moves(team);
    if depth == 1 {
        return actions.len() as u64;
    }

    for action in actions {
        let undo = board.make_move(action);
        nodes += perft(board, depth - 1, if team == 0 { 1 } else { 0 });
        board.undo_move(undo);
    }

    nodes
}

fn main() {
    let mut uci = UCICommunicator { board: Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR") };
    let stdin = io::stdin();
    let mut team = 0;
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        if line == "ucinewgame" {
            uci.board = Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
        } else if line.starts_with("position startpos moves ") {
            let moves = &line[24..].split(" ").collect::<Vec<_>>();
            uci.board = Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
            team = (moves.len() % 2) as i16;
            for action in moves {
                let action = uci.decode(action.to_string());
                uci.board.make_move(action);
            }
        } else if line.starts_with("go") {
            let mut info = create_search_info(&mut uci.board, 17);
            let results = negamax_deepening(&mut uci.board, team, 7, &mut info);
            println!("bestmove {}", uci.encode(&results.best_move.unwrap()));
        }
    }

    /*env::set_var("RUST_BACKTRACE", "FULL");
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
    let mut board = Board::load_fen(fen);
    let mut team = 0;*/

    /*println!("sadly!");
    let start = get_epoch_ms();
    let nodes = perft_psuedo(&mut board, 5, 0);
    println!("perft psuedolegal: {}", nodes);
    let end = get_epoch_ms();
    println!("time: {}ms", end - start);
    println!("nodes/ms: {}/ms", (nodes as u128) / (end - start));

    println!("-----");

    let start = get_epoch_ms();
    let nodes = perft(&mut board, 4, 0);
    println!("perft: {}", nodes);
    let end = get_epoch_ms();
    println!("time: {}ms", end - start);
    println!("nodes/ms: {}/ms", (nodes as u128) / (end - start));

    println!("-----");*/

    /*board.print_board();
    let uci = UCICommunicator { board: &mut board };

    let mut info = create_search_info(uci.board, 17);
    loop {
        let results = negamax_deepening(uci.board, team, 6, &mut info);
        let action = results.best_move.unwrap(); 
        thread::sleep(Duration::from_millis(1000));
        println!("{}", uci.encode(&action));
        uci.board.make_move(action);
        uci.board.print_board();
        team = if team == 0 { 1 } else { 0 };
        println!("-----");
    }*/
}
