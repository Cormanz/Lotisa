use crate::boards::{Board, PieceGenInfo, Action};
use super::{Piece, get_actions_delta, can_control_delta};

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