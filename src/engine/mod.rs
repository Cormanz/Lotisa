use std::{cmp::{min, max}, time::{SystemTime, UNIX_EPOCH}, ops::Neg};

use fnv::FnvHashMap;

use crate::boards::{Action, Board, PieceGenInfo, PieceInfo, in_check};

mod evaluation;
mod zobrist;
mod ordering;

pub use evaluation::*;
pub use ordering::*;
pub use zobrist::*;

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
    pub root_depth: i16
}

fn get_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}


pub fn negamax_deepening<'a>(board: &mut Board, moving_team: i16, depth: i16, info: &mut SearchInfo) -> EvaluationScore {
    let mut out = EvaluationScore { score: 0, best_move: None };
    let mut prev_nodes = 0;
    for i in 1..(depth + 1) {
        //if i != depth { continue; }
        info.root_depth = i;
        let start = get_epoch_ms();
        out = negamax_root(board, moving_team, i, info);
        let end = get_epoch_ms();
        let new_nodes = (info.quiescence_positions + info.positions) - prev_nodes;
        println!("depth {} nodes {} time {} nps {} score {} out {:?}", i, new_nodes, end - start, (new_nodes / ((end - start) + 1) as i32) * 1000, out.score, out);
        prev_nodes += new_nodes;
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
        2usize.pow((depth + 1) as u32),
        Default::default()
    );

    SearchInfo {
        positions: 0,
        quiescence_positions: 0,
        beta_cutoff: 0,
        killer_moves,
        counter_moves,
        history_moves,
        root_depth: 0,
        zobrist: generate_zobrist(board.piece_types, board.teams, board.rows * board.cols),
        transposition_table
    }
}

const MIN_SCORE: i32 = -2147483647;
const MAX_SCORE: i32 = 2147483647;
const ENDGAME_THRESHOLD: i32 = 15_000;

pub fn negamax_root(board: &mut Board, moving_team: i16, depth: i16, info: &mut SearchInfo) -> EvaluationScore {
    let evaluation = negamax(
        board,
        info,
        moving_team,
        depth,
        MIN_SCORE,
        MAX_SCORE,
    );
    //evaluation.score *= -1;
    evaluation
}

pub fn is_tactical_move(board: &mut Board, action: &Action, moving_team: i16) -> bool {
    !action.capture && !in_check(board, if moving_team == 0 { 1 } else { 0 }, board.row_gap)
}

pub fn negamax(
    board: &mut Board,
    search_info: &mut SearchInfo,
    moving_team: i16,
    mut depth: i16,
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

    let is_endgame = get_lowest_material(board, moving_team) < ENDGAME_THRESHOLD;
    let mut pv_move: Option<EvaluationScore> = None;
    let hash = hash_board(board, moving_team, &search_info.zobrist);
    let analysis = search_info.transposition_table.get(&hash);
    let mut analysis_depth = -1;
    if let Some(analysis) = analysis { 
        analysis_depth = analysis.depth;
        if analysis_depth >= depth {
            return analysis.evaluation;
        }

        if depth - analysis_depth <= 2 {
            pv_move = Some(analysis.evaluation);
        }
    }

    if pv_move.is_none() && depth > 2 {
        pv_move = Some(negamax(board, search_info, moving_team, depth - 2, alpha, beta));
    }

    if depth >= 2 {
        let bound = beta;
        let evaluation = negamax(board, search_info, moving_team, max(depth - 3 - 1, 0), -beta, 1 - beta);
        let score = -evaluation.score;
        if score >= bound {
            if is_endgame {
                // Null Move Reductions during the Endgame
                depth -= 3;
                if depth < 1 {
                    depth = 1;
                }
            } else {
                // Null Move Pruning
                return EvaluationScore {
                    score,
                    best_move: evaluation.best_move
                };
            }
        }
     }

    let mut best_move: Option<Action> = None;

    let mut best_score: i32 = MIN_SCORE;
    let base_moves = board
        .generate_legal_moves(moving_team);

    if base_moves.len() == 0 {
        if in_check(board, moving_team, board.row_gap) {
            return EvaluationScore {
                score: MIN_SCORE + 10,
                best_move: None
            };      
        }

        return EvaluationScore {
            score: 0,
            best_move: None
        };      
    }

    let mut moves: Vec<ScoredMove> = Vec::with_capacity(base_moves.len());
    for action in base_moves {
        moves.push(ScoredMove {
            action,
            score: score_move(board, depth, &action, moving_team, pv_move, search_info),
        });
    }
    moves.sort_by(|a, b| b.score.cmp(&a.score));
    search_info.positions += moves.len() as i32;
    let mut ind = 0;
    let mut working_depth = depth - 1;
    for ScoredMove { action, .. } in moves {
        search_info.beta_cutoff += 1;
        let undo = board.make_move(action);

        if action.capture {
            // Futility Pruning + Extended Futility Pruning
            if depth == 1 {
                let standing_pat = eval_board(board, moving_team);
                if standing_pat + 3000 < alpha {
                    board.undo_move(undo);
                    continue;
                }
            } else if depth == 2 {
                let standing_pat = eval_board(board, moving_team);
                if standing_pat + 5000 < alpha {
                    board.undo_move(undo);
                    continue;
                }
            }
        }
        
        let evaluation = negamax(
            board,
            search_info,
            if moving_team == 0 { 1 } else { 0 },
            working_depth,
            -beta,
            -alpha,
        );

        // Late Move Reductions
        if ind == 3 && working_depth > 0 {
            working_depth -= 1;
        }

        let score = -evaluation.score;
        board.undo_move(undo);

        if score > best_score {
            best_move = Some(action);
            best_score = score;
            if score > alpha {
                alpha = score;
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
        ind += 1;
    }

    if depth == search_info.root_depth {

    }
    let evaluation = EvaluationScore {
        score: best_score,
        best_move
    };
    if depth >= analysis_depth {
        search_info.transposition_table.insert(hash, StoredEvaluationScore { evaluation, depth });
    }
    return evaluation;
}

pub fn quiescence(
    board: &mut Board,
    search_info: &mut SearchInfo,
    moving_team: i16,
    mut alpha: i32,
    beta: i32
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

        // Delta Pruning
        let PieceInfo {
            piece_type,
            ..
        } = board.get_piece_info(action.to);
        let piece_material = board.piece_lookup.lookup(piece_type).get_material_value();
        if piece_material + 400 + standing_pat < alpha {
            continue;
        }

        // SEE Pruning
        if see(board, action.to, moving_team, Some(action.from)) < 0 {
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