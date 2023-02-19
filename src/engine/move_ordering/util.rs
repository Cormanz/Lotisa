use crate::{boards::{Action, Board}, engine::SearchInfo};

use super::MAX_KILLER_MOVES;

pub fn weigh_qs_move(search_info: &mut SearchInfo, board: &mut Board, action: &Action) -> i32 {
    if action.capture {
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

        from_material - to_material
    } else {
        let ply = ply as usize;
        let mut i = 0;
        let mut value: i32 = 0;
        while i < MAX_KILLER_MOVES && value == 0 {
            let killer = search_info.killer_moves[i][ply];
            if let Some(killer) = killer {
                if killer == *action {
                    value = 100 - (i as i32 + 1);
                }
            }
            i += 1;
        }
        value
    }
}