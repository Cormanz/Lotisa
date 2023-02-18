use crate::boards::Action;

// I like to live dangerously on the edge.
pub const MIN_VALUE: i32 = -2147483647;
pub const MAX_VALUE: i32 = 2147483647;

pub struct SearchInfo {
    pub root_depth: i16,
    pub best_move: Option<Action>
}