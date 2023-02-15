use super::{can_control_sliding, get_actions_sliding, Piece};
use crate::boards::{Action, Board, PieceGenInfo};

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
    fn get_actions(&self, board: &Board, piece_info: &PieceGenInfo, testing: bool) -> Vec<Action> {
        get_actions_sliding(&self.sliders, board, piece_info, testing)
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
            sliders: self.sliders.clone(),
        })
    }
}
