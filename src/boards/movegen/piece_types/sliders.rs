use crate::boards::{Action, ActionType, Board, PieceGenInfo};

pub fn can_control_sliding(
    sliders: &Vec<i16>,
    board: &Board,
    piece_info: &PieceGenInfo,
    targets: &Vec<i16>,
) -> bool {
    let PieceGenInfo { pos, .. } = *piece_info;
    let mut difs: Vec<i16> = Vec::with_capacity(targets.len());
    for target in targets {
        difs.push(target - pos);
    }

    for slider in sliders {
        let mut can_skip = false;
        for dif in &difs {
            if !((dif % slider) != 0 || dif.signum() != slider.signum())  {
                can_skip = true;
                break;
            }  
        }
        if can_skip { 
            continue;
        }

        let mut current_pos = pos;
        loop {
            current_pos += slider;

            match board.can_control(current_pos) {
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
                        piece_type: piece_info.piece_type,
                        capture: false,
                        info: 0,
                        team,
                    });
                }
                ActionType::CAPTURE => {
                    actions.push(Action {
                        from: pos,
                        to: current_pos,
                        piece_type: piece_info.piece_type,
                        capture: true,
                        info: 0,
                        team,
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
