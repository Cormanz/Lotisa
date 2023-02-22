use super::{base_make_move, can_control_sliding, add_actions_sliding, MakeMoveResults, Piece};
use crate::boards::{Action, Board, PieceGenInfo, StoredMove};

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
    fn add_actions(&self, actions: &mut Vec<Action>, board: &mut Board, piece_info: &PieceGenInfo) {
        add_actions_sliding(actions, &self.sliders, board, piece_info);
    }

    fn can_control(
        &self,
        board: &mut Board,
        piece_info: &PieceGenInfo,
        targets: &Vec<i16>,
    ) -> bool {
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
            sliders: self.sliders.clone(),
        })
    }
}
