use crate::boards::{Action, Board};

use super::in_check;

pub trait Restrictor {
    fn can_add(&self, board: &mut Board, action: &Action, required_team: i16) -> bool;
    fn duplicate(&self) -> Box<dyn Restrictor>;
}

pub struct DefaultRestrictor;

impl Restrictor for DefaultRestrictor {
    fn can_add(&self, board: &mut Board, action: &Action, required_team: i16) -> bool {
        if action.capture {
            let target_value = board.state[action.to as usize];
            let target_team = board.get_team(target_value);
            if board.get_piece_type(target_value, target_team) == 5 {
                return false;
            }
        }

        board.make_move(*action);
        let can_add = !in_check(board, required_team, board.row_gap);
        board.undo_move();
        return can_add;
    }

    fn duplicate(&self) -> Box<dyn Restrictor> {
        Box::new(DefaultRestrictor)
    }
}
