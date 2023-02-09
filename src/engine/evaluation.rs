use crate::boards::{Action, Board, PieceGenInfo};

pub fn eval_board(board: &Board, moving_team: i16) -> i32 {
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

    for pos in &board.pieces {
        let piece = board.state[*pos as usize];
        let team = board.get_team(piece);
        let piece_type = board.get_piece_type(piece, team);

        let piece_trait = &board.piece_lookup.lookup(piece_type);
        let piece_material = piece_trait.get_material_value();
        let team_multiplier = if team == moving_team { 1 } else { -1 };
        let piece_info = PieceGenInfo {
            pos: *pos,
            team: moving_team,
            row_gap,
            piece_type,
        };
        let center_controlled = piece_trait.can_control(board, &piece_info, &center_area);
        material += piece_material * team_multiplier;
        if center_controlled {
            center_control += team_multiplier;
        }
        if center_bigger_area.iter().any(|square| pos == square) {
            center_occupied += team_multiplier;
        }
    }

    let moves = board.generate_moves(moving_team).len() as i32;
    let opposing_moves = board
        .generate_moves(if moving_team == 0 { 1 } else { 0 })
        .len() as i32;

    material + (20 * center_control) + (5 * center_occupied) + moves - opposing_moves
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
