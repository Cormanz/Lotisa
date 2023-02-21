use crate::{boards::Action, engine::SearchInfo};

pub type HistoryMoves = Vec<Vec<Vec<i16>>>;

pub fn store_history_move(search_info: &mut SearchInfo, action: &Action, depth: i16) {
    let team = action.team as usize;
    let to = action.to as usize;
    let from = action.from as usize;

    if search_info.history_moves[team][from][to] >= 500 {
        search_info.history_moves[team][from][to] /= 2;
    }

    search_info.history_moves[team][from][to] += depth * depth;
}

pub fn get_history_move(search_info: &mut SearchInfo, action: &Action) -> i16 {
    let team = action.team as usize;
    let to = action.to as usize;
    let from = action.from as usize;

    search_info.history_moves[team][from][to]
}
