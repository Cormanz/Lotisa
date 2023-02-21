use super::{
    evaluate, get_epoch_ms, move_ordering::weigh_move, store_counter_move, store_history_move,
    weigh_qs_move, ScoredAction, SearchInfo, TranspositionEntry, MAX_VALUE, MIN_VALUE, eval,
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

        score = search(
            search_info,
            &mut uci.board,
            MIN_VALUE,
            MAX_VALUE,
            depth,
            0,
            starting_team
        );

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

pub fn search(
    search_info: &mut SearchInfo,
    board: &mut Board,
    mut alpha: i32,
    beta: i32,
    depth: i16,
    ply: i16,
    starting_team: i16
) -> i32 {
    search_info.pv_table.init_pv(ply);

    if depth == 0 {
        return evaluate(board, board.moving_team);
    }

    let actions = board.generate_moves(); // Psuedolegal Move Generation

    let mut sorted_actions: Vec<ScoredAction> = Vec::with_capacity(actions.len());
    for action in actions {
        sorted_actions.push(ScoredAction {
            action,
            score: 0
        });
    }

    let mut best_move: Option<Action> = None;
    for ScoredAction { action, .. } in sorted_actions {
        search_info.root_nodes += 1;
        if !board.is_legal(action, board.moving_team) {
            continue;
        }

        board.make_move(action);
        let score = -search(
            search_info,
            board,
            -beta,
            -alpha,
            depth - 1,
            ply + 1,
            starting_team
        );
        board.undo_move();

        if score > alpha {
            alpha = score;
            best_move = Some(action);
            search_info.pv_table.update_pv(ply, best_move);
        }
    }

    return alpha;
}
