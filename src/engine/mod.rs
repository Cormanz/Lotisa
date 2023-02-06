use crate::boards::{Board, Action, PieceGenInfo};

/*
    The engine only works for TWO-PLAYER GAMES as of now.
*/
pub fn eval_board(board: &Board, moving_team: i16) -> i32 {
    let mut material: i32 = 0;
    let mut center_control: i32 = 0;

    let row_gap = board.row_gap;

    let center_area = vec![ 66, 67, 76, 77 ];

    for pos in &board.pieces{ 
        let piece = board.state[*pos as usize];
        let team = board.get_team(piece);
        let piece_type = board.get_piece_type(piece, team);

        let piece_trait = &board.piece_map[&piece_type];
        let piece_material = piece_trait.get_material_value();
        let team_multiplier = if team == moving_team { 1 } else { -1 };
        let piece_info = PieceGenInfo {
            pos: *pos,
            team: moving_team,
            row_gap,
            piece_type
        };
        let center_controlled = piece_trait.can_control(board, &piece_info, &center_area);
        material += piece_material * team_multiplier;
        if center_controlled { center_control += team_multiplier; }
    }

    let moves = board.generate_moves(moving_team).len() as i32;
    let opposing_moves = board.generate_moves(moving_team).len() as i32;

    material + (20 * center_control) + moves - opposing_moves
}

pub fn eval_action(board: &mut Board, action: Action, moving_team: i16) -> i32 {
    let undo = board.make_move(action);
    let score = eval_board(board, moving_team);
    board.undo_move(undo);
    score
}

pub struct EvaluationScore {
    pub score: i32,
    pub best_move: Option<Action>
}

pub struct EvaluationResults {
    pub evaluation: EvaluationScore,
    pub info: SearchInfo
}

pub struct SearchInfo {
    pub positions: i32
}

pub fn negamax_root(board: &mut Board, moving_team: i16, depth: i16) ->  EvaluationResults {
    let mut info = SearchInfo { positions: 0 };
    let mut evaluation = negamax(board, &mut info, moving_team, depth, -2147483647, 2147483647);
    //evaluation.score *= -1;
    EvaluationResults {
        evaluation,
        info
    }
}

pub fn negamax(board: &mut Board, info: &mut SearchInfo, moving_team: i16, depth: i16, mut alpha: i32, beta: i32) -> EvaluationScore {
    if depth == 0 { 
        return EvaluationScore {
            score: eval_board(board, moving_team),
            best_move: None
        };
    }
    
    let mut best_move: Option<Action> = None;
    for action in board.generate_legal_moves(moving_team)  {
        info.positions += 1;
        let undo = board.make_move(action);
        let mut evaluation = negamax(board, info, if moving_team == 0 { 1 } else { 0 }, depth - 1, -beta, -alpha);
        evaluation.score *= -1;
        board.undo_move(undo);
        if evaluation.score >= beta {
            return EvaluationScore {
                score: beta,
                best_move: Some(action)
            };
        }
        if evaluation.score > alpha {
            alpha = evaluation.score;
            best_move = Some(action);
        }
    }
    return EvaluationScore {
        score: alpha,
        best_move
    };
}