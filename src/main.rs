use core::time;
use communication::UCICommunicator;
use rand::seq::{IteratorRandom, SliceRandom};
use std::{
    env, thread,
    time::{Duration, SystemTime, UNIX_EPOCH}, io::{self, BufRead, Stdin},
};

use boards::{Board};

mod boards;
mod communication;
mod utils;
mod tests;

fn test_mode() {


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
        } else if first_line == "test" {
            test_mode();
        }
        return;
    }
}