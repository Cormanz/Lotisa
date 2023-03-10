use crate::boards::{
    Action, Board, PieceGenInfo, ResetSquare, StoredMove, StoredMovePieceChange, StoredMoveType,
};

use super::{base_make_move, MakeMoveResults, Piece};

const NORMAL_MOVE: i16 = -1;
const DOUBLE_MOVE: i16 = -2;
const EN_PASSANT: i16 = -3;

fn add_promotion(board: &Board, actions: &mut Vec<Action>, action: Action, promotion_row: i16) {
    if board.get_row(action.to) == promotion_row {
        for promotion_piece_type in 0..board.piece_types {
            if promotion_piece_type == 0 {
                continue;
            }
            if promotion_piece_type == 5 {
                continue;
            }

            actions.push(Action {
                from: action.from,
                to: action.to,
                team: action.team,
                piece_type: action.piece_type,
                capture: action.capture,
                info: promotion_piece_type,
            });
        }
    } else {
        actions.push(action);
    }
}

pub struct PawnPiece;
impl Piece for PawnPiece {
    fn add_actions(&self, actions: &mut Vec<Action>, board: &mut Board, piece_info: &PieceGenInfo) {
        let PieceGenInfo {
            pos,
            row_gap,
            team,
            piece_type,
            ..
        } = *piece_info;

        let target = match team {
            0 => pos - row_gap,
            1 => pos + row_gap,
            _ => pos,
        };

        let promotion_row = match team {
            0 => board.buffer_amount,
            1 => board.rows + board.buffer_amount - 1,
            _ => board.row_gap,
        };

        let can_move_once = board.can_move(target);
        if can_move_once {
            add_promotion(
                &board,
                actions,
                Action {
                    from: pos,
                    to: target,
                    piece_type,
                    capture: false,
                    info: NORMAL_MOVE,
                    team,
                },
                promotion_row,
            );
        }

        let can_move_twice = board
            .pieces
            .iter()
            .find(|piece| piece.pos == pos)
            .unwrap()
            .first_move;

        if can_move_once && can_move_twice {
            let target = match team {
                0 => pos - row_gap * 2,
                1 => pos + row_gap * 2,
                _ => pos,
            };

            if board.can_move(target) {
                add_promotion(
                    &board,
                    actions,
                    Action {
                        from: pos,
                        to: target,
                        capture: false,
                        piece_type,
                        info: DOUBLE_MOVE,
                        team,
                    },
                    promotion_row,
                );
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
        let capture_left = board.can_capture(target_left, team);

        if capture_left {
            add_promotion(
                &board,
                actions,
                Action {
                    from: pos,
                    to: target_left,
                    piece_type,
                    capture: true,
                    info: NORMAL_MOVE,
                    team,
                },
                promotion_row,
            );
        }

        let target_right = match team {
            0 => pos - row_gap + 1,
            1 => pos + row_gap + 1,
            _ => pos,
        };
        let capture_right = board.can_capture(target_right, team);

        if capture_right {
            add_promotion(
                &board,
                actions,
                Action {
                    from: pos,
                    to: target_right,
                    piece_type,
                    capture: true,
                    info: NORMAL_MOVE,
                    team,
                },
                promotion_row,
            );
        }

        let en_passant_left = if let Some(last_move) = board.history.last() {
            let action = last_move.action;
            action.piece_type == piece_info.piece_type && action.info == -2 && action.to == pos - 1
        } else {
            false
        };

        /*
            It should be noted that theoretically, there could be an en-passant promotion in some sort of variant.
            Lotisa would not be able to support such a variant with the way it currently stores actions sadly.
        */

        if en_passant_left {
            add_promotion(
                &board,
                actions,
                Action {
                    from: pos,
                    to: target_left,
                    piece_type,
                    capture: true,
                    info: EN_PASSANT,
                    team,
                },
                promotion_row,
            );
        }

        let en_passant_right = if let Some(last_move) = board.history.last() {
            let action = last_move.action;
            action.piece_type == piece_info.piece_type && action.info == -2 && action.to == pos + 1
        } else {
            false
        };

        if en_passant_right {
            add_promotion(
                &board,
                actions,
                Action {
                    from: pos,
                    to: target_right,
                    piece_type,
                    capture: true,
                    info: EN_PASSANT,
                    team,
                },
                promotion_row,
            );
        }
    }

    fn make_move(&self, board: &mut Board, action: Action) {
        let states = vec![
            ResetSquare {
                pos: action.from,
                state: board.state[action.from as usize],
            },
            ResetSquare {
                pos: action.to,
                state: board.state[action.to as usize],
            },
        ];

        let mut pieces = vec![StoredMovePieceChange::PieceMove {
            from: action.from,
            to: action.to,
        }];

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

            let info = *board
                .pieces
                .iter()
                .find(|piece| piece.pos == en_passant_target)
                .unwrap();
            pieces.push(StoredMovePieceChange::PieceRemove { info })
        } else if action.capture {
            let info = *board
                .pieces
                .iter()
                .find(|piece| piece.pos == action.to)
                .unwrap();
            pieces.push(StoredMovePieceChange::PieceRemove { info });
        }

        base_make_move(board, action);

        let past_move = StoredMove {
            action,
            move_type: StoredMoveType::Standard { states, pieces },
        };

        board.history.push(past_move);
    }

    fn get_material_value(&self) -> i32 {
        1000
    }

    fn get_icon(&self) -> &str {
        "???"
    }

    fn duplicate(&self) -> Box<dyn Piece> {
        Box::new(PawnPiece)
    }
}
