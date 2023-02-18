use crate::boards::Action;
use std::time::{SystemTime, UNIX_EPOCH};

use super::PV;

// I like to live dangerously on the edge.
pub const MIN_VALUE: i32 = -2_000_000_000;
pub const MAX_VALUE: i32 = 2_000_000_000;
pub struct SearchInfo {
    pub root_depth: i16,
    pub time: u128,
    pub best_move: Option<Action>,
    pub pv_table: PV
}

pub fn get_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}
