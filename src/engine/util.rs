use crate::boards::Action;

// I like to live dangerously on the edge.
pub const MIN_VALUE: i32 = -2147483647;
pub const MAX_VALUE: i32 = 2147483647;

pub fn other(moving_team: i16) -> i16 {
    if moving_team == 0 {
        1
    } else {
        0
    }
}

pub struct EvaluationScore {
    score: i32,
    best_move: Option<Action>
 }