use crate::boards::Action;

// I like to live dangerously on the edge.
pub const MIN_VALUE: i32 = -2147483647;
pub const MAX_VALUE: i32 = 2147483647;

pub struct EvaluationScore {
    pub score: i32,
    pub best_move: Option<Action>
}