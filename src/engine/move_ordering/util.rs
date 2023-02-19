use crate::{boards::{Action, Board}, engine::SearchInfo};

use super::{MAX_KILLER_MOVES, see, get_history_move};

pub fn weigh_qs_move(search_info: &mut SearchInfo, board: &mut Board, action: &Action) -> i32 {
    if action.capture {
        // SEE

        if see(board, action.to, board.moving_team, Some(action.from)) < 0 {
            return -100_000;
        }

        // MVV-LVA

        let to_piece_type = board.get_piece_info(action.to).piece_type;
        
        let from_material = board.piece_lookup.lookup(action.piece_type).get_material_value();
        let to_material = board.piece_lookup.lookup(to_piece_type).get_material_value();

        from_material - to_material
    } else {
        0
    }
}

pub fn weigh_move(search_info: &mut SearchInfo, board: &mut Board, action: &Action, pv_move: &Option<Action>, ply: i16) -> i32 {
    if let Some(pv_move) = pv_move {
        if pv_move == action {
            return 1_000_000;
        }
    }

    if action.capture {
        // MVV-LVA

        let to_piece_type = board.get_piece_info(action.to).piece_type;
        
        let from_material = board.piece_lookup.lookup(action.piece_type).get_material_value();
        let to_material = board.piece_lookup.lookup(to_piece_type).get_material_value();

        let mvv_lva_score = to_material - from_material;

        if mvv_lva_score > 0 {
            100_000 + mvv_lva_score
        } else {
            -100_000 + mvv_lva_score
        }
    } else {
        let ply = ply as usize;
        let mut i = 0;
        while i < MAX_KILLER_MOVES {
            let killer = search_info.killer_moves[i][ply];
            if let Some(killer) = killer {
                if killer == *action {
                    return 10_000 - (i as i32 + 1);
                }
            }
            i += 1;
        }

        get_history_move(search_info, action) as i32
    }
}