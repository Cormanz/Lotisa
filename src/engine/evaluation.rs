use crate::boards::{Action, Board, PieceGenInfo, PieceInfo, in_check};

use super::MIN_SCORE;

pub fn weigh_move(board: &Board, a: i32, b: &Action) -> i32 {
    let PieceInfo { piece_type, .. } = board.get_piece_info(b.from);
    match piece_type {
        5 => 5,
        _ => 1
    }
}

pub fn get_lowest_material(board: &mut Board, moving_team: i16) -> i32 {
    let mut material = 0;
    let mut opposing_material = 0;

    for pos in &board.pieces {
        let pos = *pos;
        let piece = board.state[pos as usize];
        let team = board.get_team(piece);
        let piece_type = board.get_piece_type(piece, team);

        let piece_trait = &board.piece_lookup.lookup(piece_type);
        let piece_material = piece_trait.get_material_value();
        if team == moving_team {
            material += piece_material;
        } else {
            opposing_material += piece_material;
        }
    }

    if material < opposing_material {
        material
    } else {
        opposing_material
    }
}

pub fn eval_material(board: &mut Board, moving_team: i16) -> i32 {
    let mut material: i32 = 0;

    for pos in &board.pieces {
        let pos = *pos;
        let piece = board.state[pos as usize];
        let team = board.get_team(piece);
        let piece_type = board.get_piece_type(piece, team);

        let piece_trait = &board.piece_lookup.lookup(piece_type);
        let piece_material = piece_trait.get_material_value();
        let team_multiplier = if team == moving_team { 1 } else { -1 };
        material += piece_material * team_multiplier;
    }

    material
}


pub fn eval_board(board: &mut Board, moving_team: i16) -> i32 {
    let mut material: i32 = 0;
    let mut center_occupied: i32 = 0;
    let mut center_control: i32 = 0;

    let row_gap = board.row_gap;

    let center_area = vec![66, 67, 76, 77];
    let center_bigger_area = vec![
        65, 66, 67, 68, 
        75, 76, 77, 78
    ];

    let mut team_pieces: Vec<i16> = Vec::with_capacity(16);
    let mut opposing_pieces: Vec<i16> = Vec::with_capacity(16);

    for pos in &board.pieces {
        let pos = *pos;
        let piece = board.state[pos as usize];
        let team = board.get_team(piece);
        let piece_type = board.get_piece_type(piece, team);

        let piece_trait = &board.piece_lookup.lookup(piece_type);
        let piece_material = piece_trait.get_material_value();
        let team_multiplier = if team == moving_team { 1 } else { -1 };
        let piece_info = PieceGenInfo {
            pos,
            team: moving_team,
            row_gap,
            piece_type,
        };
        let center_controlled = piece_trait.can_control(board, &piece_info, &center_bigger_area);
        material += piece_material * team_multiplier;
        if center_controlled {
            center_control += team_multiplier;
        }
        if center_area.iter().any(|square| pos == *square) {
            center_occupied += (10_000 - piece_material) / 3_000;
        }

        if team == moving_team {
            team_pieces.push(pos);
        } else {
            opposing_pieces.push(pos);
        }
    }

    let base_moves = board
        .generate_legal_moves(moving_team);
        
    let moves = base_moves.iter().fold(0, |a, b| weigh_move(board, a, b));
 
    let opposing_moves = board
        .generate_moves(if moving_team == 0 { 1 } else { 0 })
        .iter().fold(0, |a, b| weigh_move(board, a, b));

    let tempo_bonus = 100;

    material + (40 * center_occupied) + (20 * center_control) + moves - opposing_moves + tempo_bonus
}

pub fn eval_action(board: &mut Board, action: Action, moving_team: i16) -> i32 {
    let undo = board.make_move(action);
    let score = eval_board(board, moving_team);
    board.undo_move(undo);
    score
}

#[derive(Clone, Copy, Debug)]
pub struct EvaluationScore {
    pub score: i32,
    pub best_move: Option<Action>,
}
