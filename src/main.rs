use core::time;
use communication::UCICommunicator;
use engine::{eval_board, negamax_root};
use rand::seq::{IteratorRandom, SliceRandom};
use std::{
    env, thread,
    time::{Duration, SystemTime, UNIX_EPOCH}, io::{self, BufRead, Stdin},
};

use boards::{Board, PieceMapLookup, AmazonPiece, Piece};

use crate::{engine::{create_search_info, negamax_deepening, SearchOptions}, boards::in_check, communication::Communicator};

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

fn test_mode() {
    env::set_var("RUST_BACKTRACE", "FULL");
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
    let mut team = 0;

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

    let mut uci = UCICommunicator { board: Board::load_fen(fen) };
    //let mut lookup = PieceMapLookup::new(PieceMapLookup::default_map(10));
    //lookup.map.insert(4, Box::new(AmazonPiece::new(10)) as Box<dyn Piece>);
    //uci.board.piece_lookup = Box::new(lookup);
    println!("DONE!");
    uci.board.print_board();

    let mut info = create_search_info(&mut uci.board, 17, SearchOptions {
        null_move_pruning: false,
        null_move_reductions: false,
        late_move_reductions_limit: 1000,
        delta_pruning: true,
        see_pruning: true,
        futility_pruning: true,
        extended_futility_pruning: true,
        move_ordering: true,
        ab_pruning: true,
        quiescience: true,
        transposition_table: true,
        pvs_search: true,
        internal_iterative_deepening: true
    });
    loop {
        let results = negamax_deepening(&mut uci.board, team, 8, &mut info, 3000);
        let action = results.best_move.unwrap(); 
        thread::sleep(Duration::from_millis(500));
        println!("{}", uci.encode(&action));
        uci.board.make_move(action);
        uci.board.print_board();
        team = if team == 0 { 1 } else { 0 };
        println!("-----");
    }
}

fn uci_mode(stdin: Stdin, a_mode: bool) {
    let mut uci = UCICommunicator { board: Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR") };
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
            let mut info = create_search_info(&mut uci.board, 17, SearchOptions {
                null_move_pruning: false,
                null_move_reductions: false,
                late_move_reductions_limit: 1000,
                delta_pruning: false,
                see_pruning: false,
                futility_pruning: false,
                extended_futility_pruning: false,
                move_ordering: true,
                ab_pruning: true,
                quiescience: false,
                transposition_table: a_mode,
                pvs_search: false,
                internal_iterative_deepening: false
            });
            let results = negamax_deepening(&mut uci.board, team, 8, &mut info, 200);
            println!("bestmove {}", uci.encode(&results.best_move.unwrap()));
        } else if line == "isready" {
            println!("readyok");
        }
    }
}

fn main() {
    let mut args = env::args().collect::<Vec<_>>();
    let stdin = io::stdin();
    let arg = &args[1];
    if arg == &"A".to_string() {
        let first_line = stdin.lock().lines().next().unwrap().unwrap();
        if first_line == "uci" {
            println!("id name Lotisa 0.0.1A");
            println!("id author Corman"); 
            println!("uciok");
            uci_mode(stdin, true);
        }
    } else if arg == &"B".to_string() {
        let first_line = stdin.lock().lines().next().unwrap().unwrap();
        if first_line == "uci" {
            println!("id name Lotisa 0.0.1B");
            println!("id author Corman"); 
            println!("uciok");
            uci_mode(stdin, false);
        }
    } else {
        let first_line = stdin.lock().lines().next().unwrap().unwrap();
        if first_line == "uci" {
            println!("id name Lotisa 0.0.1");
            println!("id author Corman"); 
            println!("uciok");
            uci_mode(stdin, true);
        } else if first_line == "test" {
            test_mode();
        }
    }
}