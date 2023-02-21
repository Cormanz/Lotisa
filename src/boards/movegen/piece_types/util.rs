use crate::boards::{Action, ActionType, Board, PieceGenInfo, StoredMove, StoredMoveType, StoredMovePieceChange, ResetSquare};

pub fn attempt_action(
    moves: &mut Vec<Action>,
    board: &Board,
    piece_info: &PieceGenInfo,
    target: i16,
) {
    let PieceGenInfo { pos, team, .. } = *piece_info;

    match board.can_move_capture(target, team) {
        ActionType::MOVE => {
            moves.push(Action {
                from: pos,
                to: target,
                piece_type: piece_info.piece_type,
                capture: false,
                info: 0,
                team,
            });
        }
        ActionType::CAPTURE => {
            moves.push(Action {
                from: pos,
                to: target,
                piece_type: piece_info.piece_type,
                capture: true,
                info: 0,
                team,
            });
        }
        ActionType::FAIL => {}
    }
}

pub struct MakeMoveResults {
    pub from_state: i16,
    pub to_state: i16,
}

pub fn base_make_move(board: &mut Board, action: Action) {
    let from_usize = action.from as usize;
    let to_usize = action.to as usize;

    let from_state = board.state[from_usize];

    board.state[to_usize] = from_state;
    board.state[from_usize] = 1;

    let to_pos_all = if action.capture {
        board.pieces.iter().position(|piece| piece.pos == action.to)
    } else {
        None
    };

    let from_pos_all = board
        .pieces
        .iter()
        .position(|piece| piece.pos == action.from)
        .unwrap();
    board.pieces[from_pos_all].pos = action.to;
    board.pieces[from_pos_all].first_move = false;

    if let Some(to_pos_all) = to_pos_all {
        board.pieces.swap_remove(to_pos_all);
    }
}

pub trait Piece {
    /*
        The default `can_control` method is not very performant. Subtraits of Piece should reimplement this for the sake of performance.
    */
    fn can_control(
        &self,
        board: &mut Board,
        piece_info: &PieceGenInfo,
        targets: &Vec<i16>,
    ) -> bool {
        let mut can_control = false;
        for action in self.get_actions(board, piece_info) {
            if targets.contains(&action.to) {
                can_control = true;
                break;
            }
        }
        can_control
    }
    fn get_actions(&self, board: &mut Board, piece_info: &PieceGenInfo) -> Vec<Action>;

    fn get_material_value(&self) -> i32;
    fn get_icon(&self) -> &str;

    fn make_move(&self, board: &mut Board, action: Action) {
        let states = vec![
            ResetSquare {
                pos: action.from,
                state: board.state[action.from as usize]
            },
            ResetSquare {
                pos: action.to,
                state: board.state[action.to as usize]
            }
        ];

        let mut pieces = vec![
            StoredMovePieceChange::PieceMove { from: action.from, to: action.to }
        ];

        if action.capture {
            let info = *board.pieces.iter().find(|piece| piece.pos == action.to).unwrap();
            pieces.push(StoredMovePieceChange::PieceRemove { info })
        }

        base_make_move(board, action);

        let past_move = StoredMove {
            action,
            move_type: StoredMoveType::Standard {
                states,
                pieces
            }
        };

        board.history.push(past_move);
    }

    fn undo_move(&self, board: &mut Board, undo: &StoredMove) {
        let StoredMove {
            action,
            move_type
        } = undo;

        match move_type {
            StoredMoveType::Standard { states, pieces } => {
                for state in states {
                    board.state[state.pos as usize] = state.state;
                }

                for piece_change in pieces {
                    match piece_change {
                        StoredMovePieceChange::PieceCreate { info } => {
                            let created_piece_index = board.pieces.iter().position(|piece| piece.pos == info.pos).unwrap();
                            board.pieces.swap_remove(created_piece_index);
                        }
                        StoredMovePieceChange::PieceRemove { info } => {
                            board.pieces.push(*info);
                        }
                        StoredMovePieceChange::PieceMove { from, to } => {
                            let to = *to;
                            let moved_piece_index = board.pieces.iter().position(|piece| piece.pos == to).unwrap();
                            board.pieces[moved_piece_index].pos = *from;
                        }
                    }
                }
            },
            StoredMoveType::Custom { state, pieces } => {
                board.state = state.clone();
                board.pieces = pieces.clone();
            }
        }
    }

    fn duplicate(&self) -> Box<dyn Piece>;
}
