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

pub fn score_active_move(board: &mut Board, depth: i16, action: &Action, moving_team: i16, tt_move: Option<EvaluationScore>, search_info: &SearchInfo) -> i32 {
    return 100_000 * see(board, action.to, moving_team, Some(action.from));
}

pub fn score_move(board: &mut Board, depth: i16, action: &Action, moving_team: i16, pv_move: Option<EvaluationScore>, search_info: &SearchInfo) -> i32 {
    let action_val = *action;

    // Order the previous best move from TT or IID first
    if let Some(pv_move) = pv_move {
        if let Some(pv_move) = pv_move.best_move {
            if pv_move == action_val {
                return 1_000_000;
            }
        }
    }

    // SEE
    if action.capture {
        return score_active_move(board, depth, action, moving_team, pv_move, search_info);
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

pub fn see(board: &mut Board, square: i16, moving_team: i16, current_attacker: Option<i16>) -> i32 {
    let row_gap = board.row_gap;
    let targets = vec![square];
    let attacking_pieces = board.pieces.iter().filter(|pos| {
        let pos = **pos;
        let PieceInfo {
            piece_type,
            team,
            ..
        } = board.get_piece_info(pos);
        if team != moving_team { return false; }
        let piece_trait = board.piece_lookup.lookup(piece_type);
        piece_trait.can_control(board, &PieceGenInfo { 
            pos,
            team,
            row_gap,
            piece_type
        }, &targets)
    }).collect::<Vec<_>>();
    if attacking_pieces.len() == 0 {
        return 0;
    }

    let attacker: i16 = if let Some(attacker) = current_attacker {
        attacker
    } else {
        let mut smallest_attacker: i16 = 0;
        let mut smallest_material: i32 = 2_000_000_000;
        for attacker in attacking_pieces {
            let pos = *attacker;
            let PieceInfo {
                piece_type,
                ..
            } = board.get_piece_info(pos);
            let piece_trait = board.piece_lookup.lookup(piece_type);
            let material = piece_trait.get_material_value();
            if material < smallest_material {
                smallest_attacker = pos;
                smallest_material = material;
            }
        }
        smallest_attacker
    };

    let PieceInfo {
        piece_type: captured_type,
        ..
    } = board.get_piece_info(square);
    let square_value = board.piece_lookup.lookup(captured_type).get_material_value();

    let undo = board.make_move(Action {
        from: attacker,
        to: square,
        capture: true,
        info: None
    });

    let value = max(0, square_value - see(board, square, if moving_team == 0 { 1 } else { 0 }, None));
    board.undo_move(undo);
    return value;

   /* skip if the square isn't attacked anymore by this side */
   /*if ( piece )
   {
      make_capture(piece, square);
      /* Do not consider captures if they lose material, therefor max zero */
      value = max (0, piece_just_captured() -see(square, other(side)) );
      undo_capture(piece, square);
   }
   return value;*/
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
        return quiescience(board, search_info, moving_team, alpha, beta); /*EvaluationScore {
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

    search_info.positions += moves.len() as i32;
    for ScoredMove { action, .. } in moves {
        search_info.beta_cutoff += 1;
        let undo = board.make_move(action);
        let evaluation = negamax(
            board,
            search_info,
            if moving_team == 0 { 1 } else { 0 },
            depth - 1,
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

pub fn quiescience(
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

    let mut best_score: i32 = standing_pat;
    if alpha < standing_pat {
        alpha = standing_pat;
    }

    let base_moves = board
        .generate_legal_moves(moving_team);
    let mut moves: Vec<ScoredMove> = Vec::with_capacity(base_moves.len());

    for action in base_moves {
        if action.capture {
            moves.push(ScoredMove {
                action,
                score: score_active_move(board, 0, &action, moving_team, None, search_info),
            });
        }
    }

    moves.sort_by(|a, b| b.score.cmp(&a.score));

    search_info.positions += moves.len() as i32;
    for ScoredMove { action, .. } in moves {
        search_info.beta_cutoff += 1;
        let undo = board.make_move(action);
        let evaluation = quiescience(
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
                        store_killer_move(action, -1, search_info);
                        let counter = search_info.counter_moves[action.from as usize][action.to as usize];
                        let mut can_counter = true;
                        if let Some(counter_move) = counter {
                            if counter_move.depth > -1 { can_counter = false; }
                        }  
                        if can_counter {
                            search_info.counter_moves[action.from as usize][action.to as usize] = Some(DepthMove { action, depth: -1 });
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
    search_info.transposition_table.insert(hash, StoredEvaluationScore { evaluation, depth: -1 });

    return evaluation;
}