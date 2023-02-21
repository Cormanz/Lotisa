use super::{can_control_delta, get_actions_delta, Piece};
use crate::boards::{Action, Board, PieceGenInfo};

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
    fn get_actions(&self, board: &mut Board, piece_info: &PieceGenInfo) -> Vec<Action> {
        get_actions_delta(&self.deltas, board, piece_info)
    }

    fn can_control(
        &self,
        board: &mut Board,
        piece_info: &PieceGenInfo,
        targets: &Vec<i16>,
    ) -> bool {
        let PieceGenInfo { pos, .. } = piece_info;
        let pos = *pos;
        let piece_row = board.get_row(pos);
        let piece_col = board.get_col(pos, piece_row);
        
        for target in targets {
            let target = *target;
            let row = board.get_row(target);
            let col = board.get_col(target, row);

            let row_dif = (piece_row - row).abs();
            let col_dif = (piece_col - col).abs();
            
            if row_dif + col_dif == 3 && row_dif != 0 && col_dif != 0 {
                return true;
            }
        }

        return false;
    }

    fn get_material_value(&self) -> i32 {
        3000
    }

    fn get_icon(&self) -> &str {
        "♞"
    }

    fn duplicate(&self) -> Box<dyn Piece> {
        Box::new(KnightPiece {
            deltas: self.deltas.clone(),
        })
    }
}
