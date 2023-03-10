use super::{can_control_sliding, add_actions_sliding, Piece};
use crate::boards::{Action, Board, PieceGenInfo};

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
        3250
    }
    fn get_icon(&self) -> &str {
        "♝"
    }

    fn duplicate(&self) -> Box<dyn Piece> {
        Box::new(BishopPiece {
            sliders: self.sliders.clone(),
        })
    }
}
