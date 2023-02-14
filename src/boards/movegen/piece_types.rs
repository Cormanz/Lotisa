use crate::boards::{Action, ActionType, Board, StoredMove};

use super::generation::PieceGenInfo;

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
                capture: false,
                info: None,
            });
        }
        ActionType::CAPTURE => {
            moves.push(Action {
                from: pos,
                to: target,
                capture: true,
                info: None,
            });
        }
        ActionType::FAIL => {}
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

        let from_usize = action.from as usize;
        let to_usize = action.to as usize;

        let from_state = board.state[from_usize];
        let to_state = board.state[to_usize];

        board.state[to_usize] = from_state;
        board.state[from_usize] = 1;

        let to_pos_all = if action.capture {
            board
                .pieces
                .iter()
                .position(|pos| *pos == action.to)
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

pub fn get_actions_delta(
    deltas: &Vec<i16>,
    board: &Board,
    piece_info: &PieceGenInfo,
) -> Vec<Action> {
    let mut actions = Vec::with_capacity(deltas.len());
    let PieceGenInfo { pos, .. } = *piece_info;
    for delta in deltas {
        attempt_action(&mut actions, board, piece_info, pos + delta);
    }
    actions
}

pub fn can_control_delta(
    deltas: &Vec<i16>,
    board: &Board,
    piece_info: &PieceGenInfo,
    targets: &Vec<i16>
) -> bool {
    let PieceGenInfo { pos, .. } = *piece_info;
    let positions = deltas.iter().map(|delta| pos + delta).collect::<Vec<_>>();
    for target in targets {
        let target_val = *target;
        if positions.contains(target) {
            match board.can_control(target_val, piece_info.team) {
                ActionType::MOVE | ActionType::CAPTURE => {
                    return true;
                }
                _ => {}
            }
        }
    }
    false
}


pub fn can_control_sliding(
    sliders: &Vec<i16>,
    board: &Board,
    piece_info: &PieceGenInfo,
    targets: &Vec<i16>,
) -> bool {
    let PieceGenInfo { pos, team, .. } = *piece_info;
    let mut difs = targets.iter().map(|target| target - pos).collect::<Vec<_>>();

    for slider in sliders {
        if difs.iter().all(|dif| (dif % slider) != 0 || dif.signum() != slider.signum()) {
            continue;
        }

        let mut current_pos = pos;
        loop {
            current_pos += slider;

            match board.can_control(current_pos, team) {
                ActionType::MOVE => {
                    if targets.contains(&current_pos) {
                        return true;
                    }
                }
                ActionType::CAPTURE => {
                    if targets.contains(&current_pos) {
                        return true;
                    }
                    break;
                }
                ActionType::FAIL => {
                    break;
                }
            }
        }
    }

    false
}

pub fn get_actions_sliding(
    sliders: &Vec<i16>,
    board: &Board,
    piece_info: &PieceGenInfo,
) -> Vec<Action> {
    let mut actions = Vec::with_capacity(sliders.len() * 2);
    let PieceGenInfo { pos, team, .. } = *piece_info;

    for slider in sliders {
        let mut current_pos = pos;
        loop {
            current_pos += slider;
            match board.can_move_capture(current_pos, team) {
                ActionType::MOVE => {
                    actions.push(Action {
                        from: pos,
                        to: current_pos,
                        capture: false,
                        info: None,
                    });
                }
                ActionType::CAPTURE => {
                    actions.push(Action {
                        from: pos,
                        to: current_pos,
                        capture: true,
                        info: None,
                    });
                    break;
                }
                ActionType::FAIL => {
                    break;
                }
            }
        }
    }

    actions
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
                    info: None,
                });
            }
        }

        let target_left = match team {
            0 => pos - row_gap - 1,
            1 => pos + row_gap - 1,
            _ => pos,
        };

        if board.can_capture(target_left, team) {
            actions.push(Action {
                from: pos,
                to: target_left,
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

pub struct KnightPiece {
    deltas: Vec<i16>,
}
impl KnightPiece {
    pub fn new(row_gap: i16) -> Self {
        KnightPiece {
            deltas: vec![
                2 * row_gap + 1,
                2 * row_gap - 1,
                -2 * row_gap + 1,
                -2 * row_gap - 1,
                row_gap + 2,
                row_gap - 2,
                -row_gap + 2,
                -row_gap - 2,
            ],
        }
    }
}

impl Piece for KnightPiece {
    fn get_actions(&self, board: &Board, piece_info: &PieceGenInfo) -> Vec<Action> {
        get_actions_delta(&self.deltas, board, piece_info)
    }

    fn can_control(&self, board: &Board, piece_info: &PieceGenInfo, targets: &Vec<i16>) -> bool {
        can_control_delta(&self.deltas, board, piece_info, targets)
    }

    fn get_material_value(&self) -> i32 {
        3000
    }

    fn get_icon(&self) -> &str {
        "♞"
    }

    fn duplicate(&self) -> Box<dyn Piece> {
        Box::new(KnightPiece {
            deltas: self.deltas.clone()
        })
    }
}

pub struct BishopPiece {
    sliders: Vec<i16>,
}
impl BishopPiece {
    pub fn new(row_gap: i16) -> Self {
        BishopPiece {
            sliders: vec![row_gap + 1, row_gap - 1, -row_gap + 1, -row_gap - 1],
        }
    }
}

impl Piece for BishopPiece {
    fn get_actions(&self, board: &Board, piece_info: &PieceGenInfo) -> Vec<Action> {
        get_actions_sliding(&self.sliders, board, piece_info)
    }

    fn can_control(&self, board: &Board, piece_info: &PieceGenInfo, targets: &Vec<i16>) -> bool {
        can_control_sliding(&self.sliders, board, piece_info, targets)
    }

    fn get_material_value(&self) -> i32 {
        3250
    }
    fn get_icon(&self) -> &str {
        "♝"
    }

    fn duplicate(&self) -> Box<dyn Piece> {
        Box::new(BishopPiece {
            sliders: self.sliders.clone()
        })
    }
}

pub struct RookPiece {
    sliders: Vec<i16>,
}
impl RookPiece {
    pub fn new(row_gap: i16) -> Self {
        RookPiece {
            sliders: vec![1, -1, row_gap, -row_gap],
        }
    }
}

impl Piece for RookPiece {
    fn get_actions(&self, board: &Board, piece_info: &PieceGenInfo) -> Vec<Action> {
        get_actions_sliding(&self.sliders, board, piece_info)
    }

    fn can_control(&self, board: &Board, piece_info: &PieceGenInfo, targets: &Vec<i16>) -> bool {
        can_control_sliding(&self.sliders, board, piece_info, targets)
    }

    fn get_material_value(&self) -> i32 {
        5000
    }

    fn get_icon(&self) -> &str {
        "♜"
    }
    
    fn duplicate(&self) -> Box<dyn Piece> {
        Box::new(RookPiece {
            sliders: self.sliders.clone()
        })
    }
}

pub struct QueenPiece {
    sliders: Vec<i16>,
}
impl QueenPiece {
    pub fn new(row_gap: i16) -> Self {
        QueenPiece {
            sliders: vec![
                1,
                -1,
                row_gap,
                -row_gap,
                row_gap + 1,
                row_gap - 1,
                -row_gap + 1,
                -row_gap - 1,
            ],
        }
    }
}

impl Piece for QueenPiece {
    fn get_actions(&self, board: &Board, piece_info: &PieceGenInfo) -> Vec<Action> {
        get_actions_sliding(&self.sliders, board, piece_info)
    }

    fn can_control(&self, board: &Board, piece_info: &PieceGenInfo, targets: &Vec<i16>) -> bool {
        can_control_sliding(&self.sliders, board, piece_info, targets)
    }

    fn get_material_value(&self) -> i32 {
        9000
    }

    fn get_icon(&self) -> &str {
        "♛"
    }

    fn duplicate(&self) -> Box<dyn Piece> {
        Box::new(QueenPiece {
            sliders: self.sliders.clone()
        })
    }
}

pub struct AmazonPiece {
    sliders: Vec<i16>,
    deltas: Vec<i16>
}
impl AmazonPiece {
    pub fn new(row_gap: i16) -> Self {
        AmazonPiece {
            sliders: vec![
                1,
                -1,
                row_gap,
                -row_gap,
                row_gap + 1,
                row_gap - 1,
                -row_gap + 1,
                -row_gap - 1,
            ],
            deltas: vec![
                2 * row_gap + 1,
                2 * row_gap - 1,
                -2 * row_gap + 1,
                -2 * row_gap - 1,
                row_gap + 2,
                row_gap - 2,
                -row_gap + 2,
                -row_gap - 2,
            ]
        }
    }
}

impl Piece for AmazonPiece {
    fn get_actions(&self, board: &Board, piece_info: &PieceGenInfo) -> Vec<Action> {
        let mut actions = get_actions_sliding(&self.sliders, board, piece_info);
        actions.extend(get_actions_delta(&self.deltas, board, piece_info));
        actions
    }

    fn can_control(&self, board: &Board, piece_info: &PieceGenInfo, targets: &Vec<i16>) -> bool {
        can_control_sliding(&self.sliders, board, piece_info, targets) || can_control_delta(&self.deltas, board, piece_info, targets)
    }

    fn get_material_value(&self) -> i32 {
        12000
    }

    fn get_icon(&self) -> &str {
        "☀︎"
    }
    
    fn duplicate(&self) -> Box<dyn Piece> {
        Box::new(AmazonPiece {
            deltas: self.deltas.clone(),
            sliders: self.sliders.clone()
        })
    }
}

pub struct KingPiece {
    deltas: Vec<i16>,
}
impl KingPiece {
    pub fn new(row_gap: i16) -> Self {
        KingPiece {
            deltas: vec![
                1,
                -1,
                row_gap,
                -row_gap,
                row_gap + 1,
                row_gap - 1,
                -row_gap + 1,
                -row_gap - 1,
            ],
        }
    }
}

impl Piece for KingPiece {
    fn get_actions(&self, board: &Board, piece_info: &PieceGenInfo) -> Vec<Action> {
        get_actions_delta(&self.deltas, board, piece_info)
    }

    fn get_icon(&self) -> &str {
        "♚"
    }

    fn get_material_value(&self) -> i32 {
        1000
    }

    fn duplicate(&self) -> Box<dyn Piece> {
        Box::new(KingPiece{ 
            deltas: self.deltas.clone()
        })
    }
}
