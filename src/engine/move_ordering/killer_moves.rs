use crate::{engine::{MAX_DEPTH, SearchInfo}, boards::Action};

pub const MAX_KILLER_MOVES: usize = 2;
pub type KillerMoves = [[Option<Action>; MAX_DEPTH as usize]; MAX_KILLER_MOVES];

pub fn store_killer_move(current_move: Action, ply: i16, search_info: &mut SearchInfo) {
    let ply = ply as usize;
    let first_killer = search_info.killer_moves[0][ply];

    // First killer must not be the same as the move being stored.
    let mut first_killer_exists = false;
    if let Some(first_killer) = first_killer {
        if first_killer == current_move {
            first_killer_exists = true;
        }
    }

    if !first_killer_exists {
        // Shift all the moves one index upward...
        for i in (1..MAX_KILLER_MOVES).rev() {
            let n = i as usize;
            let previous = search_info.killer_moves[n - 1][ply];
            search_info.killer_moves[n][ply] = previous;
        }

        // and add the new killer move in the first spot.
        search_info.killer_moves[0][ply] = Some(current_move);
    }
}