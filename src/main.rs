use core::time;
use communication::UCICommunicator;
use engine::{eval_board, negamax_root, hash_board, generate_zobrist};
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
    //uci.board.pfiece_lookup = Box::new(lookup);
    println!("DONE!");
    uci.board.print_board();

    let mut info = create_search_info(&mut uci.board, 25, vec![], SearchOptions {
        null_move_pruning: true,
        adaptive_r: true,
        null_move_reductions: true,
        late_move_reductions_limit: 2,
        late_move_reductions_upper_limit: 5,
        late_move_margin: 0, // Failed SPRT TEST
        delta_pruning: true,
        futility_pruning: true,
        extended_futility_pruning: true,
        move_ordering: true,
        ab_pruning: true,
        quiescience: true,
        transposition_table: true,
        pvs_search: true,
        internal_iterative_deepening: false,
        draw_by_repetition: false, // FAILED SPRT TEST
        quiescence_lazy_eval: false,
        pv_sort: true,
        see: true,
        killer_moves: true,
        counter_moves: true, // FAILED SPRT TEST
        history_moves: true,
        material: true,
        center_control: false,
        center_occupied: false,
        mobility: false,
        tempo_bonus: false,
        king_safety: true
    });
    loop {
        let results = negamax_deepening(&mut uci.board, team, 17, &mut info, 25000);
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
    let mut last_boards: Vec<usize> = vec![];

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        if line == "ucinewgame" {
            uci.board = Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
        } else if line.starts_with("position startpos moves ") {
            let moves = &line[24..].split(" ").collect::<Vec<_>>();
            uci.board = Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
            team = (moves.len() % 2) as i16;
            let mut team = 0;
            for action in moves {
                let action = uci.decode(action.to_string());
                uci.board.make_move(action);
                last_boards.push(hash_board(&uci.board, team, &generate_zobrist(6, 2, 64)));
                team = (team + 1) % 2;
            }
        } else if line.starts_with("go") {
            let mut info = create_search_info(&mut uci.board, 25, last_boards.clone(), SearchOptions {
                null_move_pruning: true,
                adaptive_r: true,
                null_move_reductions: true,
                late_move_reductions_limit: 2,
                late_move_reductions_upper_limit: 5,
                late_move_margin: 0, // Failed SPRT TEST
                delta_pruning: true,
                futility_pruning: true,
                extended_futility_pruning: true,
                move_ordering: true,
                ab_pruning: true,
                quiescience: true,
                transposition_table: true,
                pvs_search: true,
                internal_iterative_deepening: false,
                draw_by_repetition: false, // FAILED SPRT TEST
                quiescence_lazy_eval: false,
                pv_sort: true,
                see: true,
                killer_moves: true,
                counter_moves: true, // FAILED SPRT TEST
                history_moves: true,
                material: true,
                center_control: true,
                center_occupied: true,
                mobility: true,
                tempo_bonus: false,
                king_safety: true
            });
            let results = negamax_deepening(&mut uci.board, team, 25, &mut info, 4000);
            println!("bestmove {}", uci.encode(&results.best_move.unwrap()));
        } else if line == "isready" {
            println!("readyok");
        }
    }
}

fn main() {
    let mut args = env::args().collect::<Vec<_>>();
    let stdin = io::stdin();

    if args.len() == 1 {
        let first_line = stdin.lock().lines().next().unwrap().unwrap();
        if first_line == "uci" {
            println!("id name Lotisa 0.0.1");
            println!("id author Corman"); 
            println!("uciok");
            uci_mode(stdin, true);
        } else if first_line == "test" {
            test_mode();
        }
        return;
    }

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
    }
}