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

pub type KillerMoves = Vec<Vec<Option<Action>>>;
pub type CounterMoves = Vec<Vec<Option<DepthMove>>>;
pub type HistoryMoves = Vec<Vec<i32>>;


#[derive(Clone, Copy, Debug)]
pub struct StoredEvaluationScore {
    pub evaluation: EvaluationScore,
    pub depth: i16
}

#[derive(Clone, Copy, Debug)]
pub struct DepthMove {
    pub action: Action,
    pub depth: i16
}

pub struct SearchInfo {
    pub positions: i32,
    pub beta_cutoff: i32,
    pub killer_moves: KillerMoves,
    pub history_moves: HistoryMoves,
    pub counter_moves: CounterMoves,
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

pub fn score_move(board: &mut Board, depth: i16, action: &Action, tt_move: Option<EvaluationScore>, search_info: &SearchInfo) -> i32 {
    let action_val = *action;
    if let Some(tt_move) = tt_move {
        if let Some(tt_move) = tt_move.best_move {
            if tt_move == action_val {
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

        return 100_000 + (16 * to_material) - from_material;
    }

    // Killer Moves
    let mut i: i32 = 0;
    while i < MAX_KILLER_MOVES as i32 {
        let killer = search_info.killer_moves[i as usize][depth as usize];
        if let Some(killer) = killer {
            if action_val == killer {
                return 25_000 + (i + 1) * 4;
            }
        }
        i += 1;
    }

    let mut score = 0;

    // Counter Moves
    if let Some(counter_move) = search_info.counter_moves[action.from as usize][action.to as usize] {
        if counter_move.action == action_val {
            score += 100;
        }
    }

    // History Moves

    let history = search_info.history_moves[action.from as usize][action.to as usize];
    score += history;

    score
}

pub fn negamax_deepening<'a>(board: &mut Board, moving_team: i16, depth: i16, info: &mut SearchInfo) -> EvaluationScore {
    let mut out = EvaluationScore { score: 0, best_move: None };
    for i in 1..(depth + 1) {
        out = negamax_root(board, moving_team, i, info);
    }

    out
}

pub fn create_search_info(board: &mut Board, depth: i16) -> SearchInfo {
    let mut killer_moves: Vec<Vec<Option<Action>>> = Vec::with_capacity(MAX_KILLER_MOVES);
    for i in 0..MAX_KILLER_MOVES {
        killer_moves.insert(i, vec![None; (depth + 1) as usize]);
    }

    let positions = board.row_gap * board.col_gap;
    let mut counter_moves: Vec<Vec<Option<DepthMove>>> = Vec::with_capacity(positions as usize);
    for i in 0..(positions as usize) {
        counter_moves.insert(i, vec![None; positions as usize]);
    }

    let mut history_moves: Vec<Vec<i32>> = Vec::with_capacity(positions as usize);
    for i in 0..(positions as usize) {
        history_moves.insert(i, vec![0; positions as usize]);
    }

    let transposition_table: FnvHashMap<usize, StoredEvaluationScore> = FnvHashMap::with_capacity_and_hasher(
        4usize.pow((depth + 1) as u32),
        Default::default()
    );

    SearchInfo {
        positions: 0,
        beta_cutoff: 0,
        killer_moves,
        counter_moves,
        history_moves,
        zobrist: generate_zobrist(board.piece_types, board.teams, board.rows * board.cols),
        transposition_table
    }
}

pub fn negamax_root(board: &mut Board, moving_team: i16, depth: i16, info: &mut SearchInfo) -> EvaluationScore {
    let evaluation = negamax(
        board,
        info,
        moving_team,
        depth,
        -2147483647,
        2147483647,
    );
    //evaluation.score *= -1;
    evaluation
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
    let mut tt_move: Option<EvaluationScore> = None;
    if let Some(analysis) = analysises { 
        if analysis.depth == depth {
            return analysis.evaluation;
        }

        tt_move = Some(analysis.evaluation);
    }

    if tt_move.is_none() && depth > 2 {
        tt_move = Some(negamax(board, search_info, moving_team, depth - 2, alpha, beta));
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

    search_info.positions += moves.len() as i32;
    for ScoredMove { action, .. } in moves {
        search_info.beta_cutoff += 1;
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
                    search_info.history_moves[action.from as usize][action.to as usize] += depth as i32;

                    if !action.capture {
                        store_killer_move(action, depth, search_info);
                        let counter = search_info.counter_moves[action.from as usize][action.to as usize];
                        let mut can_counter = true;
                        if let Some(counter_move) = counter {
                            if counter_move.depth > depth { can_counter = false; }
                        }  
                        if can_counter {
                            search_info.counter_moves[action.from as usize][action.to as usize] = Some(DepthMove { action, depth });
                        }
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
