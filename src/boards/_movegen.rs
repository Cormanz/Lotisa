use super::{Board, ActionType, Action};

pub fn increment<'a>(
    board: &Board,
    moves: &mut Vec<Action>,
    incrementers: &Vec<i8>,
    start_pos: i16,
    team: i16,
) {
    for increment in incrementers {
        let mut pos = start_pos;
        loop {
            //println!("{} {}", pos, increment);
            pos = ((pos as i8) + increment) as i16;
            let state = board.state[pos as usize];
            if state == 1 {
                moves.push(Action {
                    from: start_pos,
                    to: pos,
                    capture: false,
                    info: None
                });
            } else {
                if state != 0 && board.get_team(state) != team {
                    moves.push(Action {
                        from: start_pos,
                        to: pos,
                        capture: true,
                        info: None
                    });
                }
                break;
            }
        }
    }
}

fn get_king_incrs(pos: i16, row_gap: &i16) -> [i16; 8] {
    return [
        pos + 1,
        pos - 1,
        pos + row_gap,
        pos - row_gap,
        pos + row_gap + 1,
        pos + row_gap - 1,
        pos - row_gap + 1,
        pos - row_gap - 1,
    ];
}

fn get_knight_incrs(pos: i16, row_gap: &i16) -> [i16; 8] {
    return [
        pos + row_gap + 2,
        pos + row_gap - 2,
        pos - row_gap + 2,
        pos - row_gap - 2,
        pos + (2 * row_gap) + 1,
        pos + (2 * row_gap) - 1,
        pos - (2 * row_gap) + 1,
        pos - (2 * row_gap) - 1,
    ];
}


pub fn add_moves(board: &Board, team: i16, piece_type: i16, ind: i16, row_gap: i16, i_row_gap: i8, piece: i16, moves: &mut Vec<Action>) {
    let bishop_incs = vec![i_row_gap + 1, i_row_gap - 1, -i_row_gap + 1, -i_row_gap - 1];
    let rook_incs = vec![i_row_gap, -i_row_gap, 1, -1];
    let mut queen_incs: Vec<i8> = vec![];
    queen_incs.extend(&bishop_incs);
    queen_incs.extend(&rook_incs);

    match piece_type {
        2 => {
            // PAWNS
            let pos = match team {
                0 => ind - row_gap,
                1 => ind + row_gap,
                _ => ind,
            };
            if board.can_move(pos) {
                moves.push(Action {
                    from: ind,
                    to: pos,
                    capture: false,
                    info: None
                });
            }

            match team {
                1 if ind > 30 && ind < 40 => {
                    let pos = ind + (2 * row_gap);
                    if board.can_move(pos) {
                        moves.push(Action { from: ind, to: pos, capture: false, info: None });
                    }
                }
                0 if ind > 80 && ind < 90 => {
                    let pos = ind - (2 * row_gap);
                    if board.can_move(pos) {
                        moves.push(Action { from: ind, to: pos, capture: false, info: None });
                    }
                }
                _ => {}
            }

            let pos_left = pos - 1;
            if board.can_capture(pos_left, team) {
                moves.push(Action {
                    from: ind,
                    to: pos_left,
                    capture: true,
                    info: None
                });
            }

            let pos_right = pos - 1;
            if board.can_capture(pos_right, team) {
                moves.push(Action {
                    from: ind,
                    to: pos_right,
                    capture: true,
                    info: None
                });
            }

            // TODO: Captures
        }
        3 => { // KNIGHTS
            let team = board.get_team(piece);
            for pos in get_knight_incrs(ind, &row_gap) {
                match board.can_move_capture(pos, team) {
                    ActionType::MOVE => {
                        moves.push(Action {
                            from: ind,
                            to: pos,
                            capture: false,
                            info: None
                        });
                    }
                    ActionType::CAPTURE => {
                        moves.push(Action {
                            from: ind,
                            to: pos,
                            capture: true,
                            info: None
                        });
                    }
                    ActionType::FAIL => {}
                }
                
            }
        }
        4 => { // BISHOPS
            let team = board.get_team(piece);
            increment(board, moves, &bishop_incs, ind, team);
        }
        5 => { // ROOKS
            let team = board.get_team(piece);
            increment(board, moves, &rook_incs, ind, team);
        }
        6 => { // QUEENS
            let team = board.get_team(piece);
            increment(board, moves, &queen_incs, ind, team);
        }
        7 => { // KINGS
            let team = board.get_team(piece);
            for pos in get_king_incrs(ind, &row_gap) {
                match board.can_move_capture(pos, team) {
                    ActionType::MOVE => {
                        moves.push(Action {
                            from: ind,
                            to: pos,
                            capture: false,
                            info: None
                        });
                    }
                    ActionType::CAPTURE => {
                        moves.push(Action {
                            from: ind,
                            to: pos,
                            capture: true,
                            info: None
                        });
                    }
                    ActionType::FAIL => {}
                }
            }
        }
        _ => {}
    }
}

pub fn generate_moves(board: &Board, required_team: i16) -> Vec<Action> {
    let Board { state, row_gap, pieces, .. } = board;
    let mut moves: Vec<Action> = Vec::with_capacity(64);
    let row_gap = *row_gap;
    let i_row_gap = row_gap as i8;

    for ind in pieces {
        let ind = *ind;
        let piece = state[ind as usize];
        let team = board.get_team(piece);
        if team != required_team { continue; }

        let piece_type = board.get_piece_type(piece, team);
        add_moves(board, team, piece_type, ind, row_gap, i_row_gap, piece, &mut moves);
    }

    moves
}

fn generate_legal_moves(board: &mut Board, required_team: i16) -> Vec<Action> { 
    let mut moves: Vec<Action> = Vec::with_capacity(64);

    for action in generate_moves(board, required_team) {
        let undo = board.make_move(action);
        let future_moves = generate_moves(board, if required_team == 0 { 1 } else { 0 });
        board.undo_move(undo);     

        moves.push(action);
    }

    moves
}

/*
u64 Perft(int depth)
{
  MOVE move_list[256];
  int n_moves, i;
  u64 nodes = 0;

  if (depth == 0)
    return 1ULL;

  n_moves = GenerateLegalMoves(move_list);
  for (i = 0; i < n_moves; i++) {
    MakeMove(move_list[i]);
    nodes += Perft(depth - 1);
    UndoMove(move_list[i]);
  }
  return nodes;
} */

pub fn perft(board: &mut Board, depth: i16, team: i16) -> u64 {
    let mut nodes: u64 = 0;

    let actions = generate_moves(board, team);
    if depth == 1 {
        return actions.len() as u64;
    }

    for action in actions {
        let undo = board.make_move(action);
        nodes += perft(board, depth - 1, if team == 0 { 1 } else { 0 });
        board.undo_move(undo);
    }

    return nodes;
}

pub fn b_lame(board: &mut Board, team: i16) -> u64 {
    let mut nodes: u64 = 0;

    for i in 0..500000 {
        let actions = generate_moves(board, team);
        nodes += actions.len() as u64;
    }

    return nodes;
}
