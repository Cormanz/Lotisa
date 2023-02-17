use crate::boards::{Board, PieceInfo};

pub fn evaluate(board: &mut Board, pov_team: i16) -> i32 {
    let mut score: i32 = 0;
    for piece in &board.pieces {
        let PieceInfo { piece_type, team, .. } = board.get_piece_info(piece.pos);
        let team_multiplier = if team == pov_team { 1 } else { -1 };

        let piece_trait = board.piece_lookup.lookup(piece_type);
        let material_value = piece_trait.get_material_value();
        score += team_multiplier * material_value;
    }

    score
}