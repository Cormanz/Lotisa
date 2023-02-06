use crate::boards::{Board, Action, PieceGenInfo, PieceInfo};

use self::evaluation::{EvaluationScore, eval_board};

mod evaluation;

/*
    The engine only works for TWO-PLAYER GAMES as of now.
*/

pub struct ScoredMove {
    pub action: Action,
    pub score: i32
}

pub struct EvaluationResults {
    pub evaluation: EvaluationScore,
    pub info: SearchInfo
}

pub struct SearchInfo {
    pub positions: i32
}

pub fn score_move(board: &mut Board, action: &Action) -> i32 {
    if action.capture {
        let PieceInfo { piece_type: from_piece_type, .. } = board.get_piece_info(action.from);
        let from_material = board.piece_lookup.lookup(from_piece_type).get_material_value();
        
        let PieceInfo { piece_type: to_piece_type, .. } = board.get_piece_info(action.to);
        let to_material = board.piece_lookup.lookup(to_piece_type).get_material_value();

        return 1000 * (to_material - from_material);
    }

    0
}

pub fn negamax_root(board: &mut Board, moving_team: i16, depth: i16) ->  EvaluationResults {
    let mut info = SearchInfo { positions: 0 };
    let mut evaluation = negamax(board, &mut info, moving_team, depth, -2147483647, 2147483647);
    //evaluation.score *= -1;
    EvaluationResults {
        evaluation,
        info
    }
}

pub fn negamax(board: &mut Board, info: &mut SearchInfo, moving_team: i16, depth: i16, mut alpha: i32, beta: i32) -> EvaluationScore {
    if depth == 0 { 
        return EvaluationScore {
            score: eval_board(board, moving_team),
            best_move: None
        };
    }
    
    let mut best_move: Option<Action> = None;
    let mut moves =  board.generate_legal_moves(moving_team).iter().map(|action| ScoredMove {
        action: *action,
        score: score_move(board, action)
    }).collect::<Vec<_>>();
    moves.sort_by(|a, b| b.score.cmp(&a.score));
    for ScoredMove { action, .. } in moves {
        info.positions += 1;
        let undo = board.make_move(action);
        let mut evaluation = negamax(board, info, if moving_team == 0 { 1 } else { 0 }, depth - 1, -beta, -alpha);
        evaluation.score *= -1;
        board.undo_move(undo);
        if evaluation.score >= beta {
            return EvaluationScore {
                score: beta,
                best_move: Some(action)
            };
        }
        if evaluation.score > alpha {
            alpha = evaluation.score;
            best_move = Some(action);
        }
    }
    return EvaluationScore {
        score: alpha,
        best_move
    };
}