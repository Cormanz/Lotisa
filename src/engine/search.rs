use crate::boards::{Board, Action, GameResult};
use super::{MIN_VALUE, evaluate, SearchInfo, MAX_VALUE};

pub fn search(search_info: &mut SearchInfo, board: &mut Board, mut alpha: i32, beta: i32, depth: i16, ply: i16, starting_team: i16) -> i32 {
    if depth == 0 {
        return evaluate(board, starting_team);
    }

    let actions = board.generate_legal_moves();

    /*match board.win_conditions.duplicate().compute(board, &actions) {
        GameResult::Win => {
            return MAX_VALUE - 100 + (depth as i32); // Higher Depth should mean a faster win
        }
        GameResult::Draw => {
            return 0;
        }
        GameResult::Lose => {
            return MIN_VALUE + 100 - (depth as i32); // Lower Depth should mean a slower loss
        }
        GameResult::Ongoing => {}
    }*/

    let mut best_move: Option<Action> = None;
    for action in actions {
        board.make_move(action);
        let score = -search(search_info, board, -beta, -alpha, depth - 1, ply + 1, starting_team);
        board.undo_move();

        if score > alpha {
            best_move = Some(action);
            alpha = score;

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