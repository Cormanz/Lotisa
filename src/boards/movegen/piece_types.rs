use crate::boards::{Action, ActionType, Board};

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

        if board.can_move(target) {
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
        let pawn_max_row = max_row - 1;

        let row = board.get_row(pos);

        let can_move_twice = match team {
            0 => row == pawn_max_row,
            1 => row == pawn_min_row,
            _ => false,
        };

        if can_move_twice {
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
}
