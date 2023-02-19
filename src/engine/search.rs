use crate::{boards::{Board, Action, GameResult, hash_board, in_check}, engine::store_killer_move, communication::UCICommunicator};
use super::{MIN_VALUE, evaluate, SearchInfo, MAX_VALUE, get_epoch_ms, TranspositionEntry, ScoredAction, move_ordering::{weigh_move}, weigh_qs_move, store_history_move};

pub fn root_search(search_info: &mut SearchInfo, uci: &mut UCICommunicator, starting_team: i16, max_time: u128) -> i32 {
    let mut total_time = 0;
    let mut depth = 1;
    loop {
        let start = get_epoch_ms();
        search_info.root_depth = depth;
        let score = search(search_info, &mut uci.board, MIN_VALUE, MAX_VALUE, depth, 0, starting_team, false);
        let end = get_epoch_ms();
        let time = end - start;
        total_time += time;

        search_info.time = total_time;

        let nodes = search_info.quiescence_nodes + search_info.root_nodes;
        println!(
            "info depth {} time {} score cp {} pv {} nodes {} nps {}", 
            search_info.root_depth, search_info.time, score / 10, search_info.pv_table.display_pv(uci), nodes, (nodes / (search_info.time + 1)) * 1000
        );

        if total_time >= max_time || depth >= 30 {
            return score;
        }

        depth += 1;
    }
}

pub fn quiescence(search_info: &mut SearchInfo, board: &mut Board, mut alpha: i32, beta: i32, starting_team: i16) -> i32 {
    let standing_pat = evaluate(board, board.moving_team);
    if standing_pat >= beta {
        return standing_pat;
    }

    // Delta Pruning
    if standing_pat + 9000 < alpha {
        return alpha;
    }

    if standing_pat > alpha {
        alpha = standing_pat;
    }
    

    let actions = board.generate_moves(); // Psuedolegal Move Generation

    let mut sorted_actions: Vec<ScoredAction> = Vec::with_capacity(actions.len());
    for action in actions {
        if !action.capture && !(action.piece_type == 0 && action.info >= 0) {
            board.make_move(action);
            let in_check = in_check(board, board.moving_team, board.row_gap);
            board.undo_move();

            if !in_check {
                continue;
            }
        }

        sorted_actions.push(ScoredAction {
            action,
            score: weigh_qs_move(search_info, board, &action)
        });
    }

    sorted_actions.sort_by(|a, b| b.score.cmp(&a.score));

    let mut best_move: Option<Action> = None;
    for ScoredAction { action, ..} in sorted_actions {
        search_info.quiescence_nodes += 1;
        if !board.is_legal(action, board.moving_team) { continue; }

        board.make_move(action);
        let score = -quiescence(search_info, board, -beta, -alpha, starting_team);
        board.undo_move();

        if score > alpha {
            alpha = score;
            best_move = Some(action);

            if score >= beta {
                break;
            }
        }
    }

    return alpha;
}

pub fn search(search_info: &mut SearchInfo, board: &mut Board, mut alpha: i32, beta: i32, depth: i16, ply: i16, starting_team: i16, is_pv_node: bool) -> i32 {
    search_info.pv_table.init_pv(ply);

    if depth == 0 {
        return quiescence(search_info, board, alpha, beta, starting_team);
    }

    let hash = hash_board(board, board.moving_team, &board.zobrist) % search_info.max_tt_size;
    let mut pv_move: Option<Action> = None;
    if let Some(entry) = &search_info.transposition_table[hash] {
        if entry.depth >= depth && ply < 2 && !is_pv_node {
			search_info.pv_table.update_pv(ply, entry.action);
            return entry.eval;
        }
        pv_move = entry.action;
    }

    let actions = board.generate_moves(); // Psuedolegal Move Generation

    match board.win_conditions.duplicate().compute(board, &actions) {
        GameResult::Win => {
            return MAX_VALUE - (ply as i32); // Lower Ply should mean a faster win
        }
        GameResult::Draw => {
            return 0;
        }
        GameResult::Lose => {
            return MIN_VALUE + (ply as i32); // Higher Ply should mean a slower loss
        }
        GameResult::Ongoing => {}
    }

    let mut sorted_actions: Vec<ScoredAction> = Vec::with_capacity(actions.len());
    for action in actions {
        sorted_actions.push(ScoredAction {
            action,
            score: weigh_move(search_info, board, &action, &pv_move, ply)
        });
    }

    sorted_actions.sort_by(|a, b| b.score.cmp(&a.score));

    let mut best_move: Option<Action> = None;
    let mut found_pv_node: bool = false;
    for ScoredAction { action, ..} in sorted_actions {
        search_info.root_nodes += 1;
        if !board.is_legal(action, board.moving_team) { continue; }

        board.make_move(action);
        let score = if found_pv_node {
            // Zero Window Search
            let eval = -search(search_info, board, -alpha - 1, -alpha, depth - 1, ply + 1, starting_team, false);

            if eval > alpha && eval < beta {
                // Full Window Research
                -search(search_info, board, -beta, -alpha, depth - 1, ply + 1, starting_team, true)
            } else {
                eval
            }
        } else {
            -search(search_info, board, -beta, -alpha, depth - 1, ply + 1, starting_team, true)
        };
        board.undo_move();

        if score > alpha {
            alpha = score;
            best_move = Some(action);
			search_info.pv_table.update_pv(ply, best_move);
            found_pv_node = true;

            store_history_move(search_info, &action, depth);
            if score >= beta {
                store_killer_move(search_info, &action, ply);
                break;
            }
        }
    }

    search_info.transposition_table[hash] = Some(TranspositionEntry {
        eval: alpha,
        depth,
        action: best_move
    });

    return alpha;
}