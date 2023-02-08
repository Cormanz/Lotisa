use std::cmp::max;

use fnv::FnvHashMap;

use crate::boards::{Action, Board, PieceGenInfo, PieceInfo};

use self::{
    evaluation::{eval_board, EvaluationScore},
    zobrist::{generate_zobrist, hash_board},
};

mod evaluation;
mod zobrist;

/*
    The engine only works for TWO-PLAYER GAMES as of now.
*/

const MAX_KILLER_MOVES: usize = 2;

#[derive(Clone, Copy, Debug)]
pub struct ScoredMove {
    pub action: Action,
    pub score: i32,
}

pub struct EvaluationResults {
    pub evaluation: EvaluationScore,
    pub info: SearchInfo,
}

pub type KillerMoves = Vec<Vec<Option<Action>>>;


#[derive(Clone, Copy, Debug)]
pub struct StoredEvaluationScore {
    pub evaluation: EvaluationScore,
    pub depth: i16
}

pub struct SearchInfo {
    pub positions: i32,
    pub killer_moves: KillerMoves,
    pub zobrist: Vec<usize>,
    pub transposition_table: FnvHashMap<usize, StoredEvaluationScore>,
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

pub fn score_move(board: &mut Board, depth: i16, action: &Action, tt_move: Option<&StoredEvaluationScore>, search_info: &SearchInfo) -> i32 {
    if let Some(tt_move) = tt_move {
        if let Some(tt_move) = tt_move.evaluation.best_move {
            if tt_move == *action {
                return 1_000_000;
            }
        }
    }

    // MVV-LVA
    if action.capture {
        let PieceInfo {
            piece_type: from_piece_type,
            ..
        } = board.get_piece_info(action.from);
        let from_material = board
            .piece_lookup
            .lookup(from_piece_type)
            .get_material_value();

        let PieceInfo {
            piece_type: to_piece_type,
            ..
        } = board.get_piece_info(action.to);
        let to_material = board
            .piece_lookup
            .lookup(to_piece_type)
            .get_material_value();

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

pub fn negamax_root(board: &mut Board, moving_team: i16, depth: i16) -> EvaluationResults {
    let mut killer_moves: Vec<Vec<Option<Action>>> = Vec::with_capacity(MAX_KILLER_MOVES);
    for i in 0..MAX_KILLER_MOVES {
        killer_moves.insert(i, vec![None; (depth + 1) as usize]);
    }

    let mut transposition_table: FnvHashMap<usize, StoredEvaluationScore> = FnvHashMap::with_capacity_and_hasher(
        4usize.pow((depth + 1) as u32),
        Default::default()
    );

    let mut info = SearchInfo {
        positions: 0,
        killer_moves,
        zobrist: generate_zobrist(board.piece_types, board.teams, board.rows * board.cols),
        transposition_table
    };
    let mut evaluation = negamax(
        board,
        &mut info,
        moving_team,
        depth,
        -2147483647,
        2147483647,
    );
    //evaluation.score *= -1;
    EvaluationResults { evaluation, info }
}

pub fn negamax(
    board: &mut Board,
    search_info: &mut SearchInfo,
    moving_team: i16,
    depth: i16,
    mut alpha: i32,
    mut beta: i32,
) -> EvaluationScore {
    if depth == 0 {
        return EvaluationScore {
            score: eval_board(board, moving_team),
            best_move: None,
        };
    }

    let hash = hash_board(board, moving_team, &search_info.zobrist);
    let analysises = search_info.transposition_table.get(&hash);
    let mut tt_move: Option<&StoredEvaluationScore> = None;
    if let Some(analysis) = analysises { 
        if analysis.depth == depth {
            return analysis.evaluation;
        }

        if analysis.depth > depth && analysis.evaluation.score > alpha {
            alpha = analysis.evaluation.score;
        } else if analysis.depth < depth && analysis.evaluation.score < beta {
            beta = analysis.evaluation.score;
        }

        if alpha >= beta {
            return analysis.evaluation;
        }

        tt_move = Some(analysis);
    }

    let mut best_move: Option<Action> = None;
    let mut best_score: i32 = -1000;
    let mut moves = board
        .generate_legal_moves(moving_team)
        .iter()
        .map(|action| ScoredMove {
            action: *action,
            score: score_move(board, depth, action, tt_move, search_info),
        })
        .collect::<Vec<_>>();
    moves.sort_by(|a, b| b.score.cmp(&a.score));
    //println!("{:?}", moves);

    for ScoredMove { action, .. } in moves {
        search_info.positions += 1;
        let undo = board.make_move(action);
        let mut evaluation = negamax(
            board,
            search_info,
            if moving_team == 0 { 1 } else { 0 },
            depth - 1,
            -beta,
            -alpha,
        );
        evaluation.score *= -1;
        board.undo_move(undo);

        if evaluation.score > best_score {
            best_move = Some(action);
            best_score = evaluation.score;
            if evaluation.score > alpha {
                alpha = evaluation.score;

                if evaluation.score >= beta {
                    if !action.capture {
                        store_killer_move(action, depth, search_info);
                    }
                    break;
                }
            }
        }
    }

    let evaluation = EvaluationScore {
        score: best_score,
        best_move,
    };
    search_info.transposition_table.insert(hash, StoredEvaluationScore { evaluation, depth });
    return evaluation;
}
