use crate::boards::{Action, Board, PieceGenInfo, PieceInfo, in_check};

use super::{MIN_SCORE, SearchInfo};

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


pub fn eval_board(board: &mut Board, moving_team: i16, search_info: &SearchInfo) -> i32 {
    let mut material: i32 = 0;
    let mut center_occupied: i32 = 0;
    let mut center_control: i32 = 0;

    let row_gap = board.row_gap;

    let center_area = vec![66, 67, 76, 77];
    let center_bigger_area = vec![
        65, 66, 67, 68, 
        75, 76, 77, 78
    ];

    let mut king_safety = 0;

    //let mut team_pieces: Vec<i16> = Vec::with_capacity(16);
    //let mut opposing_pieces: Vec<i16> = Vec::with_capacity(16);

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
        if search_info.options.material {
            material += piece_material * team_multiplier;
        }
        if search_info.options.center_control && center_controlled {
            center_control += team_multiplier;
        }
        if search_info.options.center_occupied && center_area.iter().any(|square| pos == *square) {
            center_occupied += (10_000 - piece_material) / 3_000;
        }

        if search_info.options.king_safety && piece_type == 5 {
            let deltas = [
                1,
                -1,
                row_gap,
                -row_gap,
                row_gap + 1,
                row_gap - 1,
                -row_gap + 1,
                -row_gap - 1,
            ];

            let mut opposing_pieces: Vec<PieceGenInfo> = Vec::with_capacity(16);
            for sub_pos in &board.pieces {
                let sub_pos = *sub_pos;
                let sub_piece = board.state[sub_pos as usize];
                let sub_team = board.get_team(sub_piece);
                if sub_team == team {
                    continue;
                }
                let sub_piece_type = board.get_piece_type(sub_piece, sub_team);
                opposing_pieces.push(PieceGenInfo { 
                    pos: sub_pos,
                    team: sub_team,
                    row_gap,
                    piece_type: sub_piece_type
                });
            }

            let mut open_squares = 0;
            let mut empty_squares = 0;

            for delta in deltas {
                let new_pos = pos + delta;
                let state = board.state[new_pos as usize];
                match state {
                    1 => {
                        empty_squares += 1;
                        open_squares += 1;
                        for sub_piece in &opposing_pieces {
                            let sub_piece_trait = board.piece_lookup.lookup(sub_piece.piece_type);
                            if sub_piece_trait.can_control(board, &sub_piece, &vec![new_pos]) {
                                open_squares -= 1;
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            }

            let blocked_squares: i32 = empty_squares - open_squares;
            let ratio = (blocked_squares.pow(2) as f32) / (empty_squares.pow(2) as f32);
            king_safety += -team_multiplier * ((2000f32 * ratio) as i32);
        }
    }

    let moves = if search_info.options.mobility {
        board
            .generate_moves(moving_team)
            .len() as i32
    } else {
        0
    };
 
    let opposing_moves = if search_info.options.mobility {
        board
            .generate_moves(if moving_team == 0 { 1 } else { 0 })
            .len() as i32
    } else {
        0
    };

    let tempo_bonus = if search_info.options.tempo_bonus { 200 } else { 0 };

    material
        + (0 * center_control)
        + (0 * center_occupied)
        + (0 * moves) - (0 * opposing_moves)
        + king_safety
        + 0
}

#[derive(Clone, Copy, Debug)]
pub struct EvaluationScore {
    pub score: i32,
    pub best_move: Option<Action>,
}
