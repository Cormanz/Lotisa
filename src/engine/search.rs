use super::{
    evaluate, get_epoch_ms, move_ordering::weigh_move, store_counter_move, store_history_move,
    weigh_qs_move, ScoredAction, SearchInfo, TranspositionEntry, MAX_VALUE, MIN_VALUE,
};
use crate::{
    boards::{hash_board, in_check, Action, Board, GameResult},
    communication::UCICommunicator,
    engine::store_killer_move,
};

pub fn root_search(
    search_info: &mut SearchInfo,
    uci: &mut UCICommunicator,
    starting_team: i16,
    max_time: u128,
) -> i32 {
    let mut total_time = 0;
    let mut depth = 1;
    let mut score: i32 = MIN_VALUE;
    loop {
        let start = get_epoch_ms();
        search_info.root_depth = depth;

        if score > MIN_VALUE {
            // Aspiration Windows

            let alpha = score - 250;
            let beta = score + 250;
            score = search(
                search_info,
                &mut uci.board,
                alpha,
                beta,
                depth,
                0,
                starting_team,
                None,
                true,
            );
            if score <= alpha || score >= beta {
                // Research
                score = search(
                    search_info,
                    &mut uci.board,
                    MIN_VALUE,
                    MAX_VALUE,
                    depth,
                    0,
                    starting_team,
                    None,
                    true,
                );
            }
        } else {
            score = search(
                search_info,
                &mut uci.board,
                MIN_VALUE,
                MAX_VALUE,
                depth,
                0,
                starting_team,
                None,
                true,
            );
        }

        let end = get_epoch_ms();
        let time = end - start;
        total_time += time;

        search_info.time = total_time;

        let nodes = search_info.quiescence_nodes + search_info.root_nodes;
        println!(
            "info depth {} time {} score cp {} nodes {} nps {} seldepth {} pv {} ",
            search_info.root_depth,
            search_info.time,
            score / 10,
            nodes,
            (nodes / (search_info.time + 1)) * 1000,
            search_info.sel_depth,
            search_info.pv_table.display_pv(uci)
        );

        if total_time >= max_time || depth >= 30 {
            return score;
        }

        depth += 1;
    }
}

pub fn quiescence(
    search_info: &mut SearchInfo,
    board: &mut Board,
    mut alpha: i32,
    beta: i32,
    starting_team: i16,
    ply: i16
) -> i32 {
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

    if search_info.sel_depth < ply {
        search_info.sel_depth = ply;
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
            score: weigh_qs_move(search_info, board, &action),
        });
    }

    sorted_actions.sort_by(|a, b| b.score.cmp(&a.score));

    let mut best_move: Option<Action> = None;
    for ScoredAction { action, .. } in sorted_actions {
        search_info.quiescence_nodes += 1;
        if !board.is_legal(action, board.moving_team) {
            continue;
        }

        board.make_move(action);
        let score = -quiescence(search_info, board, -beta, -alpha, starting_team, ply + 1);
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

pub fn search(
    search_info: &mut SearchInfo,
    board: &mut Board,
    mut alpha: i32,
    beta: i32,
    depth: i16,
    ply: i16,
    starting_team: i16,
    previous_move: Option<Action>,
    is_pv_node: bool,
) -> i32 {
    search_info.pv_table.init_pv(ply);

    if depth == 0 {
        return quiescence(search_info, board, alpha, beta, starting_team, ply);
    }

    let hash = hash_board(board, board.moving_team, &board.zobrist) % search_info.max_tt_size;
    let mut pv_move: Option<Action> = None;
    let transposition_entry = search_info.transposition_table[hash].clone();
    if let Some(entry) = &transposition_entry {
        pv_move = entry.action;
    }

    if is_pv_node && pv_move.is_none() && depth >= 4 {
        search(
            search_info,
            board,
            alpha,
            beta,
            depth - 2,
            ply,
            starting_team,
            previous_move,
            true,
        );
        pv_move = search_info.pv_table.table[ply as usize][0];
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
            score: weigh_move(search_info, board, &action, &pv_move, &previous_move, ply),
        });
    }

    sorted_actions.sort_by(|a, b| b.score.cmp(&a.score));

    if !is_pv_node && !in_check(board, board.moving_team, board.row_gap) {
        let static_eval = evaluate(board, board.moving_team);
        if depth <= 5 && static_eval - (150 * (depth as i32)) > beta {
            // Reverse Futility Pruning (Static Null Move Pruning)
            return static_eval;
        }

        if depth >= 3 {
            // Null Move Pruning

            let r = 2 + (depth / 3);
            let mut working_depth = depth - r;
            if working_depth < 0 {
                working_depth = 0;
            }

            board.moving_team = board.next_team();
            let eval = -search(
                search_info,
                board,
                -beta,
                -beta + 1,
                working_depth,
                ply + 1,
                starting_team,
                None,
                true,
            );
            board.moving_team = board.previous_team();

            if eval >= beta {
                return eval;
            }
        }
    }

    let mut best_move: Option<Action> = None;
    let mut found_pv_node: bool = false;
    let mut moves_tried = 0;
    for ScoredAction { action, .. } in sorted_actions {
        search_info.root_nodes += 1;
        if !board.is_legal(action, board.moving_team) {
            continue;
        }

        board.make_move(action);
        let score = if found_pv_node {
            let in_check = in_check(board, board.moving_team, board.row_gap);
            let is_quiet = !action.capture && !in_check;
            let mut working_depth = if !is_quiet || depth <= 2 {
                depth - 1
            } else {
                depth - 2
            };
            let static_eval = evaluate(board, board.moving_team);

            // Futility Pruning
            let fp_margin = ((working_depth as i32) * 1000) + 1000;
            if is_quiet && working_depth < 4 && static_eval + fp_margin <= alpha {
                working_depth = 0;
            }

            // Late Move Pruning
            /*if is_quiet && working_depth <= 2 && moves_tried > 2 {
                working_depth = 0;
            }*/

            if working_depth <= 0 {
                working_depth = 0;
            }
            let eval = -search(
                search_info,
                board,
                -alpha - 1,
                -alpha,
                working_depth,
                ply + 1,
                starting_team,
                Some(action),
                false,
            );

            if eval > alpha && eval < beta {
                // Full Window Research
                -search(
                    search_info,
                    board,
                    -beta,
                    -alpha,
                    depth - 1,
                    ply + 1,
                    starting_team,
                    Some(action),
                    true,
                )
            } else {
                eval
            }
        } else {
            -search(
                search_info,
                board,
                -beta,
                -alpha,
                depth - 1,
                ply + 1,
                starting_team,
                Some(action),
                true,
            )
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
                if let Some(prev_action) = previous_move {
                    //store_counter_move(search_info, prev_action, action, depth);
                }
                break;
            }
        }

        moves_tried += 1;
    }

    search_info.transposition_table[hash] = Some(TranspositionEntry {
        eval: alpha,
        depth,
        action: best_move,
    });

    return alpha;
}
