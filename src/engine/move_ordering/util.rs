use crate::{boards::{Action, Board}, engine::SearchInfo};

use super::{MAX_KILLER_MOVES, see, get_history_move, is_counter_move};

pub fn weigh_qs_move(search_info: &mut SearchInfo, board: &mut Board, action: &Action) -> i32 {
    if action.piece_type == 0 && action.info >= 0 {
        if action.info != 4 {
            return -50_000;
        }
    }

    if action.capture {
        let exchange_eval = see(board, action.to, board.moving_team, Some(action.from));
        if exchange_eval > 0 {
            100_000 + exchange_eval
        } else {
            -100_000 + exchange_eval
        }
    } else {
        0
    }
}

pub fn weigh_move(search_info: &mut SearchInfo, board: &mut Board, action: &Action, pv_move: &Option<Action>, previous_move: &Option<Action>, ply: i16) -> i32 {
    if let Some(pv_move) = pv_move {
        if pv_move == action {
            return 9_000_000;
        }
    }

    if action.piece_type == 0 && action.info >= 0 {
        if action.info != 4 {
            return -50_000;
        }
    }

    if action.capture {
        // SEE

        let exchange_eval = see(board, action.to, board.moving_team, Some(action.from));
        if exchange_eval > 0 {
            100_000 + exchange_eval
        } else {
            -100_000 + exchange_eval
        }
    } else {
        let ply = ply as usize;
        let mut i = 0;
        while i < MAX_KILLER_MOVES {
            let killer = search_info.killer_moves[i][ply];
            if let Some(killer) = killer {
                if killer == *action {
                    return 10_000 - (i as i32 + 1);
                }
            }
            i += 1;
        }

        if let Some(previous_move) = previous_move {
            if is_counter_move(search_info, previous_move, action) {
                return 2_000;
            }
        }

        get_history_move(search_info, action) as i32
    }
}