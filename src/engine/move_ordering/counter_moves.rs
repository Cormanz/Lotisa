use crate::{boards::Action, engine::SearchInfo};

pub type CounterMoves = Vec<Vec<Option<Action>>>;

pub fn store_counter_move(
    search_info: &mut SearchInfo,
    prev_action: Action,
    action: Action,
    depth: i16,
) {
    let to = prev_action.to as usize;
    let from = prev_action.from as usize;

    search_info.counter_moves[from][to] = Some(action);
}

pub fn is_counter_move(
    search_info: &mut SearchInfo,
    prev_action: &Action,
    action: &Action,
) -> bool {
    let to = prev_action.to as usize;
    let from = prev_action.from as usize;

    if let Some(counter_move) = search_info.counter_moves[to][from] {
        *action == counter_move
    } else {
        false
    }
}
