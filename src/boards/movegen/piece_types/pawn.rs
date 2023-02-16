use crate::boards::{Action, Board, PieceGenInfo, StoredMove};

use super::{base_make_move, MakeMoveResults, Piece};

const NORMAL_MOVE: i16 = -1;
const DOUBLE_MOVE: i16 = -2;
const EN_PASSANT: i16 = -3;

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
                info: NORMAL_MOVE,
                team
            });
        }

        let can_move_twice = board.pieces.iter().find(|piece| piece.pos == pos).unwrap().first_move;

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
                    info: DOUBLE_MOVE,
                    team
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
                info: NORMAL_MOVE,
                team
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
                info: NORMAL_MOVE,
                team
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
                info: EN_PASSANT,
                team
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
                info: EN_PASSANT,
                team
            });
        }

        actions
    }

    fn make_move(&self, board: &mut Board, action: Action) {
        let old_pieces = board.pieces.clone();
        let old_state = if action.info == -3 { Some(board.state.clone()) } else { None };


        if action.info == EN_PASSANT {
            /*
                The action in Lotisa's "to" represents where the capturer needs to go, not the piece that needs to be captured.
                Since we're doing en passant, we'll always know which way to modify the row of the "to" to find the captured piece.
                Then, we move the captured piece to the "to" square, and simulate a normal capture.
                We store the undo with the normal action to make sure squares get reset normally, though.
                Perhaps this is a more efficient way to do this, but I thought this was easiest.
            */

            let en_passant_target = if action.team == 0 {
                action.to + board.row_gap
            } else {
                action.to - board.row_gap
            };

            let en_passant_target_usize = en_passant_target as usize;
            let en_passant_target_state = board.state[en_passant_target_usize];
            let to_usize = action.to as usize;
            board.state[to_usize] = en_passant_target_state;
            board.state[en_passant_target_usize] = 1;
        }

        let MakeMoveResults {
            from_state,
            to_state,
        } = base_make_move(board, action);

        let past_move = StoredMove {
            action,
            from_previous: from_state,
            to_previous: to_state,
            pieces: old_pieces,
            state: old_state
        };

        board.history.push(past_move);
    }

    fn get_material_value(&self) -> i32 {
        1000
    }

    fn get_icon(&self) -> &str {
        "♙"
    }

    fn duplicate(&self) -> Box<dyn Piece> {
        Box::new(PawnPiece)
    }
}
