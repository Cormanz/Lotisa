use std::{cmp::{min, max}, time::{SystemTime, UNIX_EPOCH}, ops::Neg, collections::HashMap};

use fnv::FnvHashMap;

use crate::{boards::{Action, Board, PieceGenInfo, PieceInfo, in_check}, communication::Communicator};

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
    pub root_depth: i16,
    pub options: SearchOptions,
    pub last_boards: Vec<usize>
}

pub struct SearchOptions {
    pub null_move_pruning: bool,
    pub adaptive_r: bool,
    pub null_move_reductions: bool,
    pub late_move_reductions_limit: i16,
    pub late_move_reductions_upper_limit: i16,
    pub late_move_margin: i32,
    pub futility_pruning: bool,
    pub extended_futility_pruning: bool,
    pub delta_pruning: bool,
    pub move_ordering: bool,
    pub ab_pruning: bool,
    pub quiescience: bool,
    pub pvs_search: bool,
    pub transposition_table: bool,
    pub internal_iterative_deepening: bool,
    pub draw_by_repetition: bool,
    pub quiescence_lazy_eval: bool,
    pub pv_sort: bool,
    pub see: bool,
    pub killer_moves: bool,
    pub counter_moves: bool,
    pub history_moves: bool,
    pub mobility: bool,
    pub tempo_bonus: bool,
    pub center_control: bool,
    pub center_occupied: bool,
    pub king_safety: bool,
    pub material: bool
}

fn get_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub fn is_draw_by_repetition(last_boards: &Vec<usize>) -> bool {
    let len = last_boards.len();
    if len < 3 {
        false
    } else {
        last_boards[len - 1] == last_boards[len - 3]
    }
}

pub fn negamax_deepening<'a>(board: &mut Board, moving_team: i16, depth: i16, info: &mut SearchInfo, max_time: u128) -> EvaluationScore {
    let mut out = EvaluationScore { score: 0, best_move: None };
    let mut prev_nodes = 0;
    let mut total_time = 0;
    for i in 1..(depth + 1) {
        //if i != depth { continue; }
        info.root_depth = i;
        let start = get_epoch_ms();
        out = negamax_root(board, moving_team, i, info);
        let end = get_epoch_ms();
        let new_nodes = (info.quiescence_positions + info.positions) - prev_nodes;
        let time = end - start;
        println!("info depth {} nodes {} time {} nps {} score cp {}", 
            i, new_nodes, end - start, (new_nodes / (time + 1) as i32) * 1000, out.score / 10
        );
        prev_nodes += new_nodes;

        total_time += time;
        if total_time > max_time {
            return out;
        }
    }

    out
}

pub fn create_search_info(board: &mut Board, depth: i16, last_boards: Vec<usize>, options: SearchOptions) -> SearchInfo {
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
        transposition_table,
        options,
        last_boards
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
        None,
        false
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
    prev_action: Option<Action>,
    is_pv: bool
) -> EvaluationScore {
    if depth == 0 {
        return if search_info.options.quiescience {
            quiescence(board, search_info, moving_team, alpha, beta)
        } else {
            EvaluationScore {
                score: eval_board(board, moving_team, &search_info),
                best_move: None
            }
        }
    }

    let lowest_material = get_lowest_material(board, moving_team);
    let is_endgame = lowest_material < ENDGAME_THRESHOLD;
    let mut pv_move: Option<EvaluationScore> = None;
    let hash = hash_board(board, moving_team, &search_info.zobrist);
    let analysis = search_info.transposition_table.get(&hash);
    let mut analysis_depth = -1;

    if search_info.options.draw_by_repetition && is_draw_by_repetition(&search_info.last_boards) {
        return EvaluationScore {
            score: 0,
            best_move: None
        };   
    }
    
    if let Some(analysis) = analysis { 
        analysis_depth = analysis.depth;
        if analysis_depth >= depth {
            return analysis.evaluation;
        }

        if depth - analysis_depth <= 2 {
            pv_move = Some(analysis.evaluation);
        }
    }

    if pv_move.is_none() && depth >= 4 && search_info.options.internal_iterative_deepening {
        pv_move = Some(negamax(board, search_info, moving_team, depth - 2, alpha, beta, prev_action, is_pv));
    }
    
    let in_check = in_check(board, moving_team, board.row_gap) ;
    if depth >= 3 && !in_check && !is_pv {
        let mut working_depth = if search_info.options.adaptive_r {
            depth / 2
        } else {
            depth - 2
        };
        if working_depth < 0 {
            working_depth = 0;
        }
        if working_depth > depth - 2 {
            working_depth = depth - 2;
        }
        let evaluation = negamax(
            board, search_info, 
            if moving_team == 0 { 1 } else { 0 }, 
            working_depth, -beta, 
            -beta + 1, None, false
        );
        let score = -evaluation.score;
        if score >= beta {
            // Null Move Reductions
            if (depth >= 5 || is_endgame) && search_info.options.null_move_reductions {
                return EvaluationScore {
                    score,
                    best_move: None
                }
            } else {
                depth = max(0, depth - 4);

                if depth == 0 {
                    return if search_info.options.quiescience {
                        quiescence(board, search_info, moving_team, alpha, beta)
                    } else {
                        EvaluationScore {
                            score: eval_board(board, moving_team, &search_info),
                            best_move: None
                        }
                    }
                }
            }
        }
    }

    if !in_check && !is_pv {
        // Futility Pruning + Extended Futility Pruning
        if search_info.options.futility_pruning && depth == 1 {
            let standing_pat = eval_material(board, moving_team);
            if standing_pat + 2000 < alpha {
                return EvaluationScore {
                    score: standing_pat,
                    best_move: None
                };
            }
        } else if search_info.options.extended_futility_pruning && depth == 2 {
            let standing_pat = eval_material(board, moving_team);
            if standing_pat + 4000 < alpha {
                return EvaluationScore {
                    score: standing_pat,
                    best_move: None
                };
            }
        }
    }

    let mut best_move: Option<Action> = None;

    let mut best_score: i32 = MIN_SCORE;
    let base_moves = board
        .generate_legal_moves(moving_team);

    if base_moves.len() == 0 {
        if in_check {
            let evaluation = EvaluationScore {
                score: MIN_SCORE + 100 - (depth as i32),
                best_move: None
            };      
            if depth >= analysis_depth {
                search_info.transposition_table.insert(hash, StoredEvaluationScore { evaluation, depth });
            }
            return evaluation; 
        }

        let evaluation = EvaluationScore {
            score: 0,
            best_move: None
        };     
        if depth >= analysis_depth {
            search_info.transposition_table.insert(hash, StoredEvaluationScore { evaluation, depth });
        }
        return evaluation; 
    }

    let mut moves: Vec<ScoredMove> = Vec::with_capacity(base_moves.len());
    for action in base_moves {
        moves.push(ScoredMove {
            action,
            score: score_move(board, depth, &action, &prev_action, moving_team, pv_move, search_info),
        });
    }

    if search_info.options.move_ordering {
        moves.sort_by(|a, b| b.score.cmp(&a.score));
    }

    search_info.positions += moves.len() as i32;
    let mut ind = 0;
    let working_depth = depth - 1;
    let mut b_search_pv = false;
    for ScoredMove { action, .. } in moves {
        search_info.beta_cutoff += 1;
        let undo = board.make_move(action);
        search_info.last_boards.push(hash_board(&board, moving_team, &search_info.zobrist));
        
        let evaluation = if b_search_pv && search_info.options.pvs_search {
            let late_move_reductions = ind >= search_info.options.late_move_reductions_limit;
            let late_move_reductions_upper = ind >= search_info.options.late_move_reductions_upper_limit;
            let evaluation = negamax(
                board,
                search_info,
                if moving_team == 0 { 1 } else { 0 },
                 if late_move_reductions_upper {
                    max((2 * working_depth) / 3, 0)
                } else if late_move_reductions {
                    max(working_depth - 1, 0)
                } else {
                    working_depth
                },
                -alpha - 1,
                -alpha,
                Some(action),
                true
            );
            let mut score = -evaluation.score;
            if late_move_reductions {
                score += search_info.options.late_move_margin; // Since we're searching at depth - 1, let's add an additional margin.
            }

            if -evaluation.score > alpha && -evaluation.score < beta {
                negamax(
                    board,
                    search_info,
                    if moving_team == 0 { 1 } else { 0 },
                    working_depth,
                    -beta,
                    -alpha,
                    Some(action),
                    false
                )
            } else {
                evaluation
            }
        } else {
            negamax(
                board,
                search_info,
                if moving_team == 0 { 1 } else { 0 },
                working_depth,
                -beta,
                -alpha,
                Some(action),
                false
            )
        };

        let score = -evaluation.score;
        search_info.last_boards.pop();
        board.undo_move(undo);

        if score > best_score {
            best_move = Some(action);
            best_score = score;
            if score > alpha {
                b_search_pv = true;
                alpha = score;
                if score >= beta {                
                    search_info.history_moves[action.from as usize][action.to as usize] += ((depth * depth) + 2) as i32;

                    if !action.capture {
                        store_killer_move(action, depth, search_info);
                        let counter = search_info.counter_moves[action.from as usize][action.to as usize];
                        let mut can_counter = true;
                        if let Some(counter_move) = counter {
                            if counter_move.depth > depth { can_counter = false; }
                        }  
                        if can_counter {
                            if let Some(prev_action) = prev_action {
                                search_info.counter_moves[prev_action.from as usize][prev_action.to as usize] = Some(DepthMove { action, depth });
                            }
                        }
                    }
                    if search_info.options.ab_pruning {
                        break;
                    }
                }
            }
        }
        ind += 1;
    }

    let evaluation = EvaluationScore {
        score: best_score,
        best_move
    };
    if depth >= analysis_depth && search_info.options.transposition_table {
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

    let standing_pat = eval_board(board, moving_team, &search_info);
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
        if !action.capture {
            let undo = board.make_move(action);
            let in_check = in_check(board, moving_team, board.row_gap);
            board.undo_move(undo);
            if !in_check {
                continue;
            }
        }

        if action.capture {
            if search_info.options.delta_pruning {
                // Delta Pruning
                let PieceInfo {
                    piece_type,
                    ..
                } = board.get_piece_info(action.to);
                let piece_material = board.piece_lookup.lookup(piece_type).get_material_value();
                if piece_material + 400 + standing_pat < alpha {
                    continue;
                }
            }
        }

        moves.push(ScoredMove {
            action,
            score: score_qs_move(board, 0, &action, moving_team, None, search_info),
        });
    }

    if search_info.options.move_ordering {
        moves.sort_by(|a, b| b.score.cmp(&a.score));
    }

    search_info.quiescence_positions += moves.len() as i32;

    for ScoredMove { action, .. } in moves {
        search_info.beta_cutoff += 1;
        let undo = board.make_move(action);
        search_info.last_boards.push(hash_board(board, moving_team, &search_info.zobrist));
        let evaluation = quiescence(
            board,
            search_info,
            if moving_team == 0 { 1 } else { 0 },
            -beta,
            -alpha,
        );
        let score = -evaluation.score;
        search_info.last_boards.pop();
        board.undo_move(undo);

        if score > best_score {
            best_move = Some(action);
            best_score = score;
            if score > alpha {
                alpha = score;
                if score >= beta {                
                    search_info.history_moves[action.from as usize][action.to as usize] += 1;
                    if search_info.options.ab_pruning {
                        break;
                    }
                }
            }
        }
    }

    let evaluation = EvaluationScore {
        score: best_score,
        best_move
    };
    if search_info.options.transposition_table {
        search_info.transposition_table.insert(hash, StoredEvaluationScore { evaluation, depth: 0 });
    }

    return evaluation;
}