use std::cmp::max;

use crate::boards::{Board, Action, PieceGenInfo, PieceInfo};

use self::evaluation::{EvaluationScore, eval_board};

mod evaluation;

/*
    The engine only works for TWO-PLAYER GAMES as of now.
*/

const MAX_KILLER_MOVES: usize = 2;

pub struct ScoredMove {
    pub action: Action,
    pub score: i32
}

pub struct EvaluationResults {
    pub evaluation: EvaluationScore,
    pub info: SearchInfo
}

pub type KillerMoves = Vec<Vec<Option<Action>>>;

pub struct SearchInfo {
    pub positions: i32,
    pub killer_moves: KillerMoves
}

pub fn store_killer_move(current_move: Action, depth_left: i16, search_info: &mut SearchInfo) {
    let depth = depth_left as usize;
    let first_killer = search_info.killer_moves[0][depth];

    // First killer must not be the same as the move being stored.
    let mut can_shift = true;
    if let Some(first_killer) = first_killer {
        can_shift = first_killer != current_move;
    }

    if can_shift {
        // Shift all the moves one index upward...
        for i in (1..MAX_KILLER_MOVES).rev() {
            let n = i as usize;
            let previous = search_info.killer_moves[n - 1][depth];
            search_info.killer_moves[n][depth] = previous;
        }

        // and add the new killer move in the first spot.
        search_info.killer_moves[0][depth] = Some(current_move);
    }
}

pub fn score_move(board: &mut Board, depth: i16, action: &Action, search_info: &SearchInfo) -> i32 {
    // MVV-LVA
    if action.capture {
        let PieceInfo { piece_type: from_piece_type, .. } = board.get_piece_info(action.from);
        let from_material = board.piece_lookup.lookup(from_piece_type).get_material_value();
        
        let PieceInfo { piece_type: to_piece_type, .. } = board.get_piece_info(action.to);
        let to_material = board.piece_lookup.lookup(to_piece_type).get_material_value();

        return 1000 + (16 * to_material) - from_material;
    }

    // Killer Moves
    let mut i: i32 = 0;
    let action_val = *action;
    while i < MAX_KILLER_MOVES as i32 {
        let killer = search_info.killer_moves[i as usize][depth as usize];
        if let Some(killer) = killer {
            if action_val == killer {
                return 500 + (i + 1) * 4;
            }
        }
        i += 1;
    }

    0
}

pub fn negamax_root(board: &mut Board, moving_team: i16, depth: i16) ->  EvaluationResults {
    let mut killer_moves: Vec<Vec<Option<Action>>> = Vec::with_capacity(MAX_KILLER_MOVES);
    for i in 0..MAX_KILLER_MOVES {
        killer_moves.insert(i, vec![None; (depth + 1) as usize]);
    }
    let mut info = SearchInfo { positions: 0, killer_moves };
    let mut evaluation = negamax(board, &mut info, moving_team, depth, -2147483647, 2147483647);
    //evaluation.score *= -1;
    EvaluationResults {
        evaluation,
        info
    }
}

pub fn negamax(board: &mut Board, search_info: &mut SearchInfo, moving_team: i16, depth: i16, mut alpha: i32, beta: i32) -> EvaluationScore {
    if depth == 0 { 
        return EvaluationScore {
            score: eval_board(board, moving_team),
            best_move: None
        };
    }
    
    let mut best_move: Option<Action> = None;
    let mut moves =  board.generate_legal_moves(moving_team).iter().map(|action| ScoredMove {
        action: *action,
        score: score_move(board, depth, action, search_info)
    }).collect::<Vec<_>>();
    moves.sort_by(|a, b| b.score.cmp(&a.score));
    for ScoredMove { action, .. } in moves {
        search_info.positions += 1;
        let undo = board.make_move(action);
        let mut evaluation = negamax(board, search_info, if moving_team == 0 { 1 } else { 0 }, depth - 1, -beta, -alpha);
        evaluation.score *= -1;
        board.undo_move(undo);

        if evaluation.score >= beta {
            if !action.capture {
                store_killer_move(action, depth, search_info);
            }

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