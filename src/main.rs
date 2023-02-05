use core::time;
use std::{
    env,
    time::{SystemTime, UNIX_EPOCH, Duration}, thread,
};
use engine::{eval_action, negamax_root};
use rand::seq::{SliceRandom, IteratorRandom};

use boards::Board;

use crate::engine::eval_board;

mod boards;
mod engine;

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
    env::set_var("RUST_BACKTRACE", "FULL");
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
    let mut board = Board::load_fen(fen);
    let mut team = 0;

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

    println!("-----");

    loop {
        let moves = board.generate_legal_moves(team);
        let start = get_epoch_ms();
        let results = negamax_root(&mut board, team, 5);
        let end = get_epoch_ms();
        let action = results.evaluation.best_move.unwrap(); /*if moves.iter().any(|action| action.capture) {
            moves.iter().filter(|action| action.capture).choose(&mut rand::thread_rng()).unwrap()
        } else {
            moves.choose(&mut rand::thread_rng()).unwrap()
        };*/
        thread::sleep(Duration::from_millis(1700));
        let positions = results.info.positions;
        println!("time: {}ms ({positions} nodes)", end - start);
        println!("nodes/ms: {}", positions / (end - start) as i32);
        println!("move score: {} for {}", results.evaluation.score, team);
        board.make_move(action);
        board.print_board();
        team = if team == 0 { 1 } else { 0 };
        println!("-----");
    }
}
