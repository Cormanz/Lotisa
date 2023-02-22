use crate::boards::{Action, ActionType, Board, PieceGenInfo};

use super::attempt_action;

pub fn add_actions_delta(
    actions: &mut Vec<Action>,
    deltas: &Vec<i16>,
    board: &Board,
    piece_info: &PieceGenInfo,
) {
    let PieceGenInfo { pos, .. } = *piece_info;
    for delta in deltas {
        attempt_action(actions, board, piece_info, pos + delta);
    }
}

pub fn can_control_delta(
    deltas: &Vec<i16>,
    board: &Board,
    piece_info: &PieceGenInfo,
    targets: &Vec<i16>,
) -> bool {
    let PieceGenInfo { pos, .. } = *piece_info;
    let positions = deltas.iter().map(|delta| pos + delta).collect::<Vec<_>>();
    for target in targets {
        let target_val = *target;
        if positions.contains(target) {
            match board.can_control(target_val) {
                ActionType::MOVE | ActionType::CAPTURE => {
                    return true;
                }
                _ => {}
            }
        }
    }
    false
}
