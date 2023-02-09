use std::cmp::min;

use fnv::FnvHashMap;

use crate::boards::{Action, Board, PieceGenInfo, PieceInfo};

use self::{
    evaluation::{eval_board, EvaluationScore},
    zobrist::{generate_zobrist, hash_board}, ordering::{score_move, score_qs_move, MAX_KILLER_MOVES, see, store_killer_move},
};

mod evaluation;
mod zobrist;
mod ordering;

/*
    The engine only works for TWO-PLAYER GAMES as of now.
*/

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
    pub quiescence_positions: i32,
    pub beta_cutoff: i32,
    pub killer_moves: KillerMoves,
    pub history_moves: HistoryMoves,
    pub counter_moves: CounterMoves,
    pub zobrist: Vec<usize>,
    pub transposition_table: FnvHashMap<usize, StoredEvaluationScore>,
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
        quiescence_positions: 0,
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
        return quiescence(board, search_info, moving_team, alpha, beta); 
         /*EvaluationScore {
            score: eval_board(board, moving_team),
            best_move: None,
        };*/
    }

    let mut pv_move: Option<EvaluationScore> = None;
    let hash = hash_board(board, moving_team, &search_info.zobrist);
    let analysis = search_info.transposition_table.get(&hash);
    if let Some(analysis) = analysis { 
        if analysis.depth >= depth {
            return analysis.evaluation;
        }

        pv_move = Some(analysis.evaluation);
    }

    if pv_move.is_none() && depth > 2 {
        pv_move = Some(negamax(board, search_info, moving_team, depth - 2, alpha, beta));
    }

    let mut best_move: Option<Action> = None;
    let mut best_score: i32 = -100_000_000;
    let base_moves = board
        .generate_legal_moves(moving_team);
    let mut moves: Vec<ScoredMove> = Vec::with_capacity(base_moves.len());
    for action in base_moves {
        moves.push(ScoredMove {
            action,
            score: score_move(board, depth, &action, moving_team, pv_move, search_info),
        });
    }
    moves.sort_by(|a, b| b.score.cmp(&a.score));
    //println!("{:?}", moves);

    let mut b_search_pv = true;
    search_info.positions += moves.len() as i32;
    for ScoredMove { action, .. } in moves {
        search_info.beta_cutoff += 1;
        let undo = board.make_move(action);
        
        let evaluation = if b_search_pv || depth < 2 {
            negamax(
                board,
                search_info,
                if moving_team == 0 { 1 } else { 0 },
                depth - 1,
                -beta,
                -alpha,
            )
        } else {
            let evaluation = negamax(
                board,
                search_info,
                moving_team, // Skipping opponent's turn: null move
                depth - min(3, depth),
                -alpha - 1,
                -alpha,
            );
            if -evaluation.score > alpha {
                negamax(
                    board,
                    search_info,
                    if moving_team == 0 { 1 } else { 0 },
                    depth - 1,
                    -beta,
                    -alpha,
                )          
            } else {
                board.undo_move(undo);
                continue;
            }
        };

        let score = -evaluation.score;
        board.undo_move(undo);

        if score > best_score {
            best_move = Some(action);
            best_score = score;
            if score > alpha {
                alpha = score;
                b_search_pv = false;
                if score >= beta {                
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

pub fn quiescence(
    board: &mut Board,
    search_info: &mut SearchInfo,
    moving_team: i16,
    mut alpha: i32,
    mut beta: i32
) -> EvaluationScore {
    let hash = hash_board(board, moving_team, &search_info.zobrist);
    let analysis = search_info.transposition_table.get(&hash);
    if let Some(analysis) = analysis { 
        return analysis.evaluation;
    }

    let mut best_move: Option<Action> = None;
    let standing_pat = eval_board(board, moving_team);

    if standing_pat >= beta {
        return EvaluationScore {
            score: beta,
            best_move: None
        };
    }

    let mut best_score: i32 = standing_pat;
    if standing_pat > alpha {
        alpha = standing_pat;
    }

    let base_moves = board
        .generate_legal_moves(moving_team);
    let mut moves: Vec<ScoredMove> = Vec::with_capacity(base_moves.len());

    for action in base_moves {
        if !action.capture { continue; }

        // Quiescence SEE Futility Pruning
        let PieceInfo {
            piece_type,
            ..
        } = board.get_piece_info(action.to);
        let piece_material = board.piece_lookup.lookup(piece_type).get_material_value();
        if piece_material + 400 + standing_pat < alpha {
            continue;
        }

        moves.push(ScoredMove {
            action,
            score: score_qs_move(board, 0, &action, moving_team, None, search_info),
        });
    }

    moves.sort_by(|a, b| b.score.cmp(&a.score));

    search_info.quiescence_positions += moves.len() as i32;

    for ScoredMove { action, .. } in moves {
        search_info.beta_cutoff += 1;
        let undo = board.make_move(action);
        let evaluation = quiescence(
            board,
            search_info,
            if moving_team == 0 { 1 } else { 0 },
            -beta,
            -alpha,
        );
        let score = -evaluation.score;
        board.undo_move(undo);

        if score > best_score {
            best_move = Some(action);
            best_score = score;
            if score > alpha {
                alpha = score;
                if score >= beta {                
                    search_info.history_moves[action.from as usize][action.to as usize] += 1;
                    if !action.capture {
                        let counter = search_info.counter_moves[action.from as usize][action.to as usize];
                        let mut can_counter = true;
                        if let Some(counter_move) = counter {
                            if counter_move.depth > 0 { can_counter = false; }
                        }  
                        if can_counter {
                            search_info.counter_moves[action.from as usize][action.to as usize] = Some(DepthMove { action, depth: 0 });
                        }
                    }
                    break;
                }
            }
        }
    }

    let evaluation = EvaluationScore {
        score: best_score,
        best_move
    };
    search_info.transposition_table.insert(hash, StoredEvaluationScore { evaluation, depth: 0 });

    return evaluation;
}