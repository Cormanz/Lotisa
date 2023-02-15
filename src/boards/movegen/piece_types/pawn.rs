use crate::boards::{Action, PieceGenInfo, Board};

use super::Piece;

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
                info: None,
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
                    info: None,
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

        if board.can_capture(target_left, team) {
            actions.push(Action {
                from: pos,
                to: target_left,
                piece_type: piece_info.piece_type,
                capture: true,
                info: None,
            });
        }

        let target_right = match team {
            0 => pos - row_gap + 1,
            1 => pos + row_gap + 1,
            _ => pos,
        };

        if board.can_capture(target_right, team) {
            actions.push(Action {
                from: pos,
                to: target_right,
                piece_type: piece_info.piece_type,
                capture: true,
                info: None,
            });
        }

        actions
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
