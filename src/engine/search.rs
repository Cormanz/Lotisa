use crate::boards::{Board, Action};
use super::{MIN_VALUE, evaluate, SearchInfo};

pub fn search(search_info: &mut SearchInfo, board: &mut Board, mut alpha: i32, beta: i32, depth: i16, ply: i16, starting_team: i16) -> i32 {
    if depth == 0 {
        return evaluate(board, starting_team);
    }

    let actions = board.generate_legal_moves();
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