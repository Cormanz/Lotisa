use crate::{boards::Action, engine::SearchInfo};

#[derive(Clone, Copy)]
pub struct CounterMovesInfo {
    depth: i16,
    action: Action
}
pub type CounterMoves = Vec<Vec<Option<CounterMovesInfo>>>;

pub fn store_counter_move(
    search_info: &mut SearchInfo,
    prev_action: Action,
    action: Action,
    depth: i16
) {
    let to = prev_action.to as usize;
    let from = prev_action.from as usize;

    if let Some(counter_move) = search_info.counter_moves[from][to] {
        if counter_move.depth > depth {
            return;
        }
    }
    search_info.counter_moves[from][to] = Some(CounterMovesInfo { depth, action });
}

pub fn is_counter_move(
    search_info: &mut SearchInfo,
    prev_action: &Action,
    action: &Action,
) -> bool {
    let to = prev_action.to as usize;
    let from = prev_action.from as usize;

    if let Some(counter_move) = search_info.counter_moves[to][from] {
        *action == counter_move.action
    } else {
        false
    }
}
