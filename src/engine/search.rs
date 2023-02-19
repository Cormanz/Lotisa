use crate::boards::{Board, Action, GameResult};
use super::{MIN_VALUE, evaluate, SearchInfo, MAX_VALUE, get_epoch_ms};

pub fn root_search(search_info: &mut SearchInfo, board: &mut Board, starting_team: i16, max_time: u128) -> i32 {
    let mut total_time = 0;
    let mut depth = 1;
    loop {
        let start = get_epoch_ms();
        search_info.root_depth = depth;
        let score = search(search_info, board, MIN_VALUE, MAX_VALUE, depth, 0, starting_team);
        let end = get_epoch_ms();
        let time = end - start;
        total_time += time;

        search_info.time = total_time;
        if total_time >= max_time {
            return score;
        }

        depth += 1;
    }
}

pub fn search(search_info: &mut SearchInfo, board: &mut Board, mut alpha: i32, beta: i32, depth: i16, ply: i16, starting_team: i16) -> i32 {
    search_info.pv_table.init_pv(ply);

    if depth == 0 {
        return evaluate(board, starting_team);
    }

    let actions = board.generate_moves(); // Psuedolegal Move Generation

    match board.win_conditions.duplicate().compute(board, &actions) {
        GameResult::Win => {
            return MAX_VALUE - (ply as i32); // Lower Ply should mean a faster win
        }
        GameResult::Draw => {
            return 0;
        }
        GameResult::Lose => {
            return MIN_VALUE + (ply as i32); // Higher Ply should mean a slower loss
        }
        GameResult::Ongoing => {}
    }

    let mut best_move: Option<Action> = None;
    for action in actions {
        search_info.search_nodes += 1;
        if !board.is_legal(action, board.moving_team) { continue; }

        board.make_move(action);
        let score = -search(search_info, board, -beta, -alpha, depth - 1, ply + 1, starting_team);
        board.undo_move();

        if score > alpha {
            best_move = Some(action);
            alpha = score;
			search_info.pv_table.update_pv(ply, Some(action));

            if score >= beta {
                break;
            }
        }
    }

    if ply == 0 {
        search_info.best_move = best_move;
    }
    return alpha;
}