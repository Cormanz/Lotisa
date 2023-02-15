use crate::boards::{Action, ActionType, Board, PieceGenInfo, StoredMove};

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
            });
        }
        ActionType::CAPTURE => {
            moves.push(Action {
                from: pos,
                to: target,
                piece_type: piece_info.piece_type,
                capture: true,
                info: 0,
            });
        }
        ActionType::FAIL => {}
    }
}

pub struct MakeMoveResults {
    pub from_state: i16,
    pub to_state: i16,
}

pub fn base_make_move(board: &mut Board, action: Action) -> MakeMoveResults {
    let from_usize = action.from as usize;
    let to_usize = action.to as usize;

    let from_state = board.state[from_usize];
    let to_state = board.state[to_usize];

    board.state[to_usize] = from_state;
    board.state[from_usize] = 1;

    let to_pos_all = if action.capture {
        board.pieces.iter().position(|pos| *pos == action.to)
    } else {
        None
    };

    let from_pos_all = board
        .pieces
        .iter()
        .position(|pos| *pos == action.from)
        .unwrap();
    board.pieces[from_pos_all] = action.to;

    if let Some(to_pos_all) = to_pos_all {
        board.pieces.swap_remove(to_pos_all);
    }

    MakeMoveResults {
        from_state,
        to_state,
    }
}

pub trait Piece {
    /*
        The default `can_control` method is not very performant. Subtraits of Piece should reimplement this for the sake of performance.
    */
    fn can_control(&self, board: &Board, piece_info: &PieceGenInfo, targets: &Vec<i16>) -> bool {
        let mut can_control = false;
        for action in self.get_actions(board, piece_info) {
            if targets.contains(&action.to) {
                can_control = true;
                break;
            }
        }
        can_control
    }
    fn get_actions(&self, board: &Board, piece_info: &PieceGenInfo) -> Vec<Action>;

    fn get_material_value(&self) -> i32;
    fn get_icon(&self) -> &str;

    fn make_move(&self, board: &mut Board, action: Action) {
        let old_pieces = board.pieces.clone();

        let MakeMoveResults {
            from_state,
            to_state,
        } = base_make_move(board, action);

        let past_move = StoredMove {
            action,
            from_previous: from_state,
            to_previous: to_state,
            pieces: old_pieces,
        };

        board.history.push(past_move);
    }

    fn undo_move(&mut self, board: &mut Board, undo: StoredMove) {
        let StoredMove {
            action,
            to_previous,
            from_previous,
            pieces,
        } = undo;
        board.state[action.to as usize] = to_previous;
        board.state[action.from as usize] = from_previous;
        board.pieces = pieces;
    }

    fn duplicate(&self) -> Box<dyn Piece>;
}
