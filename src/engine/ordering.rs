use crate::boards::{Action, Board, PieceInfo, PieceGenInfo, in_check};
use std::cmp::max;

use super::{evaluation::EvaluationScore, SearchInfo, eval_board};

pub const MAX_KILLER_MOVES: usize = 2;

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
    let see_val = see(board, action.to, moving_team, Some(action.from));
    if see_val > 0 {
        100_000 + see_val
    } else {
        -1_000_000
    }
}

pub fn score_qs_move(board: &mut Board, depth: i16, action: &Action, moving_team: i16, pv_move: Option<EvaluationScore>, search_info: &SearchInfo) -> i32 {
    // SEE
    if action.capture {
        return score_active_move(board, depth, action, moving_team, pv_move, search_info);
    }

    0    
}


pub fn score_move(board: &mut Board, depth: i16, action: &Action, prev_action: &Option<Action>, moving_team: i16, pv_move: Option<EvaluationScore>, search_info: &SearchInfo) -> i32 {
    let mut score = 0;
    let action_val = *action;

    if search_info.options.pv_sort {
        // Order the previous best move from TT or IID first
        if let Some(pv_move) = pv_move {
            if let Some(pv_move) = pv_move.best_move {
                if pv_move == action_val {
                    return 100_000_000;
                }
            }
        }
    }

    // SEE
    if search_info.options.see && action.capture {
        return score_active_move(board, depth, action, moving_team, pv_move, search_info);
    }

    // Killer Moves
    let mut i: i32 = 0;
    if search_info.options.killer_moves {
        while i < MAX_KILLER_MOVES as i32 {
            let killer = search_info.killer_moves[i as usize][depth as usize];
            if let Some(killer) = killer {
                if action_val == killer {
                    return 50_000 + (i + 1) * 4;
                }
            }
            i += 1;
        }
    }

    /*let undo = board.make_move(action_val);
    if in_check(board, moving_team, board.row_gap) {
        score += 100;
    }
    board.undo_move(undo);*/

    // History Moves

    let mut score = 0;
    if search_info.options.history_moves {
        let history = search_info.history_moves[action.from as usize][action.to as usize];
        score = history;
    }

    
    // Counter Moves
    if search_info.options.counter_moves {
        if let Some(prev_action) = prev_action {
            if let Some(counter_move) = search_info.counter_moves[prev_action.from as usize][prev_action.to as usize] {
                if counter_move.action == action_val {
                    return 100;
                }
            }
        }
    }

    return score;
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

    let value = square_value - see(board, square, if moving_team == 0 { 1 } else { 0 }, None);
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