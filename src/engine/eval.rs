use crate::boards::{generate_legal_moves, generate_moves, Action, Board, PieceGenInfo, PieceInfo};

const INNER_CENTER_SQUARES: [i16; 4] = [54, 55, 64, 65];

const CENTER_SQUARES: [i16; 16] = [
    43, 44, 45, 46, 53, 54, 55, 56, 63, 64, 65, 66, 73, 74, 75, 76,
];

pub fn weigh_mobility_move(board: &mut Board, action: &Action) -> i32 {
    let mut score = 15;

    if CENTER_SQUARES.contains(&action.to) {
        if INNER_CENTER_SQUARES.contains(&action.to) {
            score += 5;
        } else {
            score += 3;
        }
    }

    if action.capture {
        let attacker_material = board
            .piece_lookup
            .lookup(action.piece_type)
            .get_material_value();
        let victim_piece_type = board.get_piece_info(action.to).piece_type;
        let victim_material = board
            .piece_lookup
            .lookup(victim_piece_type)
            .get_material_value();
        if victim_material > attacker_material {
            score += 30;
        }
    }

    score
}

pub fn evaluate(board: &mut Board, pov_team: i16) -> i32 {
    let mut score: i32 = 0;
    let row_gap = board.row_gap;

    for piece in board.pieces.clone() {
        let PieceInfo {
            piece_type, team, ..
        } = board.get_piece_info(piece.pos);
        let team_multiplier = if team == pov_team { 1 } else { -1 };

        let piece_trait = board.piece_lookup.lookup(piece_type);
        let material_value = piece_trait.get_material_value();
        score += team_multiplier * material_value;
    }

    score
}
