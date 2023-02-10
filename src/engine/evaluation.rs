use crate::boards::{Action, Board, PieceGenInfo, PieceInfo, in_check};

use super::MIN_SCORE;

pub fn weigh_move(board: &Board, a: i32, b: &Action) -> i32 {
    let PieceInfo { piece_type, .. } = board.get_piece_info(b.from);
    if piece_type == 5 {
        a + 5
    } else {
        a + 1
    }
}

pub fn get_lowest_material(board: &mut Board, moving_team: i16) -> i32 {
    let mut material = 0;
    let mut opposing_material = 0;
    let row_gap = board.row_gap;

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
            opposing_material += opposing_material;
        }
    }

    if material < opposing_material {
        material
    } else {
        opposing_material
    }
}

pub fn eval_board(board: &mut Board, moving_team: i16) -> i32 {
    let mut material: i32 = 0;
    let mut center_occupied: i32 = 0;
    let mut center_control: i32 = 0;

    let row_gap = board.row_gap;

    let center_area = vec![66, 67, 76, 77];
    let center_bigger_area = vec![
        55, 56, 57, 58,
        65, 66, 67, 68, 
        75, 76, 77, 78,
        85, 86, 87, 88
    ];

    let mut synergy = 0;
    let mut opposing_synergy = 0;

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
        let center_controlled = piece_trait.can_control(board, &piece_info, &center_area);
        material += piece_material * team_multiplier;
        if center_controlled {
            center_control += team_multiplier;
        }
        if center_bigger_area.iter().any(|square| pos == *square) {
            center_occupied += team_multiplier;
        }

        if team == moving_team {
            team_pieces.push(pos);
        } else {
            opposing_pieces.push(pos);
        }
    }
    
    let borrowed_team_pieces = &team_pieces;
    let borrowed_opposing_pieces = &opposing_pieces;
    for pos in borrowed_team_pieces {
        let pos = *pos;
        let team = board.get_team(pos);
        let piece_type = board.get_piece_type(pos, team);
        let piece_info = PieceGenInfo {
            pos,
            team: moving_team,
            row_gap,
            piece_type,
        };
        let piece_trait = board.piece_lookup.lookup(piece_type);
        if piece_trait.can_control(board, &piece_info, borrowed_team_pieces) {
            synergy += 1;
        }
    }

    
    for pos in borrowed_opposing_pieces {
        let pos = *pos;
        let team = board.get_team(pos);
        let piece_type = board.get_piece_type(pos, team);
        let piece_info = PieceGenInfo {
            pos,
            team: moving_team,
            row_gap,
            piece_type,
        };
        let piece_trait = board.piece_lookup.lookup(piece_type);
        if piece_trait.can_control(board, &piece_info, borrowed_opposing_pieces) {
            opposing_synergy += 1;
        }
    }

    let base_moves = board
        .generate_legal_moves(moving_team);
        
    let moves = base_moves.iter().fold(0, |a, b| weigh_move(board, a, b));
 
    let opposing_moves = board
        .generate_moves(if moving_team == 0 { 1 } else { 0 })
        .iter().fold(0, |a, b| weigh_move(board, a, b));

    material + (2 * synergy) - (2 * opposing_synergy) + (20 * center_control) + (12 * center_occupied) + moves - opposing_moves
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
