use crate::boards::{Action, Board, PieceGenInfo, StoredMove};

use super::{base_make_move, MakeMoveResults, Piece};

pub enum PawnMoveInfo {
    NormalMove,
    DoubleMove,
    EnPassant,
    Promotion(i16),
}

fn get_info(info: i16) -> PawnMoveInfo {
    match info {
        -1 => PawnMoveInfo::NormalMove,
        -2 => PawnMoveInfo::DoubleMove,
        -3 => PawnMoveInfo::EnPassant,
        piece_type => PawnMoveInfo::Promotion(piece_type),
    }
}

pub struct PawnPiece;
impl Piece for PawnPiece {
    fn get_actions(&self, board: &Board, piece_info: &PieceGenInfo) -> Vec<Action> {
        let mut actions = Vec::with_capacity(2);

        let PieceGenInfo {
            pos, row_gap, team, ..
        } = *piece_info;

        let target = match team {
            0 => pos - row_gap,
            1 => pos + row_gap,
            _ => pos,
        };

        let can_move_once = board.can_move(target);
        if can_move_once {
            actions.push(Action {
                from: pos,
                to: target,
                piece_type: piece_info.piece_type,
                capture: false,
                info: -1,
            });
        }

        let min_row = board.buffer_amount;
        let max_row = board.rows + board.buffer_amount;

        let pawn_min_row = min_row + 1;
        let pawn_max_row = max_row - 2;

        let row = board.get_row(pos);

        let can_move_twice = match team {
            0 => row == pawn_max_row,
            1 => row == pawn_min_row,
            _ => false,
        };

        if can_move_once && can_move_twice {
            let target = match team {
                0 => pos - row_gap * 2,
                1 => pos + row_gap * 2,
                _ => pos,
            };

            if board.can_move(target) {
                actions.push(Action {
                    from: pos,
                    to: target,
                    capture: false,
                    piece_type: piece_info.piece_type,
                    info: -2,
                });
            }
        }

        /*
        TODO: EN PASSANT

        Restrictions:
            - The enemy pawn moved 2 squares forward in a previous move
            - This pawn is right next to it

        If both of those are true, we can take the pawn by moving to where it would've been 1 square from there
        */

        let target_left = match team {
            0 => pos - row_gap - 1,
            1 => pos + row_gap - 1,
            _ => pos,
        };
        let capture_left = board.can_capture(target_left, team) ;

        if capture_left {
            actions.push(Action {
                from: pos,
                to: target_left,
                piece_type: piece_info.piece_type,
                capture: true,
                info: -1,
            });
        }

        let target_right = match team {
            0 => pos - row_gap + 1,
            1 => pos + row_gap + 1,
            _ => pos,
        };
        let capture_right = board.can_capture(target_right, team);

        if capture_right {
            actions.push(Action {
                from: pos,
                to: target_right,
                piece_type: piece_info.piece_type,
                capture: true,
                info: -1,
            });
        }

        let en_passant_left = if let Some(last_move) = board.history.last() {
            let action = last_move.action;
            action.piece_type == piece_info.piece_type && action.info == -2 && action.to == pos - 1
        } else {
            false
        };

        if en_passant_left {
            actions.push(Action {
                from: pos,
                to: target_left,
                piece_type: piece_info.piece_type,
                capture: true,
                info: -3
            });
        }

        let en_passant_right = if let Some(last_move) = board.history.last() {
            let action = last_move.action;
            action.piece_type == piece_info.piece_type && action.info == -2 && action.to == pos + 1
        } else {
            false
        };

        if en_passant_right {
            actions.push(Action {
                from: pos,
                to: target_right,
                piece_type: piece_info.piece_type,
                capture: true,
                info: -3
            });
        }

        actions
    }

    fn make_move(&self, board: &mut Board, action: Action) {
        let old_pieces = board.pieces.clone();

        let new_action = if action.info == -3 {
            let shifted_to = action.to - board.row_gap;
            let shifted_to_usize = shifted_to as usize;
            let to_usize = action.to as usize;
            let to_state = board.state[to_usize];
            board.state[to_usize] = 1;
            board.state[shifted_to_usize] = to_state;

            let shifted_action = Action {
                from: action.from,
                to: shifted_to,
                capture: true,
                piece_type: action.piece_type,
                info: -3,
            };

            shifted_action
        } else {
            action
        };

        let MakeMoveResults {
            from_state,
            to_state,
        } = base_make_move(board, new_action);

        let past_move = StoredMove {
            action,
            from_previous: from_state,
            to_previous: to_state,
            pieces: old_pieces,
        };

        board.history.push(past_move);
    }

    fn get_material_value(&self) -> i32 {
        1000
    }

    fn get_icon(&self) -> &str {
        "♟"
    }

    fn duplicate(&self) -> Box<dyn Piece> {
        Box::new(PawnPiece)
    }
}
