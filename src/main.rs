use communication::UCICommunicator;
use tests::boards_bench;
use core::time;
use rand::seq::{IteratorRandom, SliceRandom};
use std::{
    env,
    io::{self, BufRead, Stdin},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use boards::Board;

use crate::cli::run_uci;

mod boards;
mod cli;
mod communication;
mod engine;
mod tests;
mod util;

fn main() {
    boards_bench();
    return;

    env::set_var("RUST_BACKTRACE", "1");

    let mut args = env::args().collect::<Vec<_>>();
    let stdin = io::stdin();

    if args.len() == 1 {
        let first_line = stdin.lock().lines().next().unwrap().unwrap();
        if first_line == "uci" {
            println!("id name Lotisa 0.0.1");
            println!("id author Corman");
            println!("uciok");
            run_uci(stdin);
        } else if first_line == "test" {
        }
        return;
    }
}
