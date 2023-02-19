use crate::boards::Action;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{PV, KillerMoves};

#[derive(Clone)]
pub struct TranspositionEntry {
    pub eval: i32,
    pub depth: i16
}

#[derive(Clone)]
pub struct ScoredAction {
    pub action: Action,
    pub score: i32
}

// I like to live dangerously on the edge.
pub const MIN_VALUE: i32 = -2_000_000_000;
pub const MAX_VALUE: i32 = 2_000_000_000;

pub const MAX_DEPTH: usize = 100;

pub struct SearchInfo {
    pub search_nodes: u128,
    pub root_depth: i16,
    pub time: u128,
    pub pv_table: PV,
    pub transposition_table: Vec<Option<TranspositionEntry>>,
    pub max_tt_size: usize,
    pub killer_moves: KillerMoves
}

pub fn get_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}
