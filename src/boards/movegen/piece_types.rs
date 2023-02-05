use crate::boards::{Action, Board, ActionType};

use super::generation::PieceGenInfo;

pub fn attempt_action(moves: &mut Vec<Action>, board: &Board, piece_info: &PieceGenInfo, target: i16) {
    let PieceGenInfo { pos, team, .. } = *piece_info; 

    match board.can_move_capture(target, team) {
        ActionType::MOVE => {
            moves.push(Action {
                from: pos,
                to: target,
                capture: false,
                info: None
            });
        }
        ActionType::CAPTURE => {
            moves.push(Action {
                from: pos,
                to: target,
                capture: true,
                info: None
            });
        }
        ActionType::FAIL => {}
    }
}

pub trait Piece {
    /*
        The default `can_attack` method is not very performant. Subtraits of Piece should reimplement this for the sake of performance.
    */
    fn can_attack(&self, board: &Board, piece_info: &PieceGenInfo, target: i16) -> bool {
        self.get_actions(board, piece_info).iter().any(|action| action.to == target)
    }
    fn get_actions(&self, board: &Board, piece_info: &PieceGenInfo) -> Vec<Action>;

    fn get_icon(&self) -> &str;
}

fn get_actions_delta(deltas: &Vec<i16>, board: &Board, piece_info: &PieceGenInfo) -> Vec<Action> {
    let mut actions = Vec::new();
    let PieceGenInfo { pos, .. } = *piece_info;
    for delta in deltas {
        attempt_action(&mut actions, board, piece_info, pos + delta);
    }
    actions
}
    
fn can_attack_sliding(sliders: &Vec<i16>, board: &Board, piece_info: &PieceGenInfo, target: i16) -> bool {
    let PieceGenInfo { pos, team, .. } = *piece_info;
    let dif = target - pos;
    let dif_signum = dif.signum();

    for slider in sliders {
        if dif_signum != slider.signum() { continue; }
        if (dif % slider) != 0 { continue; }
        
        let mut current_pos = pos;
        loop {
            current_pos += slider;

            match board.can_move_capture(current_pos, team) {
                ActionType::MOVE => {
                    if current_pos == target { return true; }
                }
                ActionType::CAPTURE => {
                    if current_pos == target { return true; }
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

fn get_actions_sliding(sliders: &Vec<i16>, board: &Board, piece_info: &PieceGenInfo) -> Vec<Action> {
    let mut actions = Vec::new();
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
                        info: None
                    });
                }
                ActionType::CAPTURE => {
                    actions.push(Action {
                        from: pos,
                        to: current_pos,
                        capture: true,
                        info: None
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

        let PieceGenInfo { pos, row_gap, team, .. } = *piece_info;

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
                info: None
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
            _ => false
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
                    info: None
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
                info: None
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
                info: None
            });
        }

        actions

    }

    fn can_attack(&self, board: &Board, piece_info: &PieceGenInfo, target: i16) -> bool {
        let PieceGenInfo { pos, row_gap, team, .. } = *piece_info;

        let target_left = match team {
            0 => pos - row_gap - 1,
            1 => pos + row_gap - 1,
            _ => pos,
        };    

        let target_right = match team {
            0 => pos - row_gap + 1,
            1 => pos + row_gap + 1,
            _ => pos,
        };     
        
        (target_left == target && board.can_capture(target_left, team)) || 
        (target_right == target && board.can_capture(target_right, team))
    }

    fn get_icon(&self) -> &str {
        "♟"
    }
}

pub struct KnightPiece {
    deltas: Vec<i16>
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
                -row_gap - 2
            ]
        }
    }
}

impl Piece for KnightPiece {
    fn get_actions(&self, board: &Board, piece_info: &PieceGenInfo) -> Vec<Action> {
        get_actions_delta(&self.deltas, board, piece_info)
    }
    fn get_icon(&self) -> &str {
        "♞"
    }
}

pub struct BishopPiece {
    sliders: Vec<i16>
}
impl BishopPiece {
    pub fn new(row_gap: i16) -> Self {
        BishopPiece {
            sliders: vec![
                row_gap + 1,
                row_gap - 1,
                -row_gap + 1,
                -row_gap - 1
            ]
        }
    }
}

impl Piece for BishopPiece {
    fn get_actions(&self, board: &Board, piece_info: &PieceGenInfo) -> Vec<Action> {
         get_actions_sliding(&self.sliders, board, piece_info)
    }

    fn can_attack(&self, board: &Board, piece_info: &PieceGenInfo, target: i16) -> bool {
        can_attack_sliding(&self.sliders, board, piece_info, target)
    }

    fn get_icon(&self) -> &str {
        "♝"
    }
}

pub struct RookPiece {
    sliders: Vec<i16>
}
impl RookPiece {
    pub fn new(row_gap: i16) -> Self {
        RookPiece {
            sliders: vec![
                1,
                -1,
                row_gap,
                -row_gap
            ]
        }
    }
}

impl Piece for RookPiece {
    fn get_actions(&self, board: &Board, piece_info: &PieceGenInfo) -> Vec<Action> {
        get_actions_sliding(&self.sliders, board, piece_info)
    }

    fn can_attack(&self, board: &Board, piece_info: &PieceGenInfo, target: i16) -> bool {
        can_attack_sliding(&self.sliders, board, piece_info, target)
    }
    fn get_icon(&self) -> &str {
        "♜"
    }
}

pub struct QueenPiece {
    sliders: Vec<i16>
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
                -row_gap - 1
            ]
        }
    }
}

impl Piece for QueenPiece {
    fn get_actions(&self, board: &Board, piece_info: &PieceGenInfo) -> Vec<Action> {
        get_actions_sliding(&self.sliders, board, piece_info)
    }

    fn can_attack(&self, board: &Board, piece_info: &PieceGenInfo, target: i16) -> bool {
        can_attack_sliding(&self.sliders, board, piece_info, target)
    }

    fn get_icon(&self) -> &str {
        "♛"
    }
}

pub struct KingPiece {
    deltas: Vec<i16>
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
                -row_gap - 1
            ]
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
}