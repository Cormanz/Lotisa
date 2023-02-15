use super::{can_control_delta, get_actions_delta, Piece};
use crate::boards::{Action, Board, PieceGenInfo};

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
        Box::new(KingPiece {
            deltas: self.deltas.clone(),
        })
    }
}
