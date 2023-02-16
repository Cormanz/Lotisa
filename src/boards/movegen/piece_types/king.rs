use super::{can_control_delta, get_actions_delta, Piece, base_make_move, MakeMoveResults};
use crate::boards::{Action, Board, PieceGenInfo, ActionType, PieceInfo, StoredMove, PersistentPieceInfo};

const NORMAL_MOVE: i16 = 0;
const CASTLING_MOVE: i16 = 1;

fn get_actions_castling(
    sliders: &Vec<i16>,
    board: &Board,
    piece_info: &PieceGenInfo
) -> Vec<Action> {
    let mut actions = Vec::with_capacity(sliders.len() * 2);
    let PieceGenInfo { pos, team, piece_type, .. } = *piece_info;

    let mut opposing_pieces: Vec<&PersistentPieceInfo> = Vec::with_capacity(16);
    for piece in &board.pieces {
        if piece.pos == pos && !piece.first_move {
            return actions;
        }

        let PieceInfo { team: piece_team, .. } = board.get_piece_info(piece.pos);
        if piece_team != team {
            opposing_pieces.push(&piece);
        }
    }

    let row_gap = board.row_gap;

    for slider in sliders {
        let mut current_pos = pos;
        loop {
            current_pos += slider;
            match board.state[current_pos as usize] {
                0 => { break; }
                1 => {
                    let targets = vec![ current_pos ];
                    let can_be_attacked = opposing_pieces.iter().any(|piece| {
                        let PieceInfo { team: attacker_team, piece_type: attacker_piece_type, .. } = board.get_piece_info(piece.pos);
                        let piece_trait = board.piece_lookup.lookup(piece_info.piece_type);
                        let piece_gen_info = &PieceGenInfo {
                            pos: piece.pos,
                            team: attacker_team,
                            row_gap,
                            piece_type: attacker_piece_type
                        };
                        piece_trait.can_control(board, piece_gen_info, &targets)
                    });
                    if can_be_attacked {
                        break;
                    }
                }
                _ => {
                    let PieceInfo { team: target_team, piece_type: target_piece_type, .. } = board.get_piece_info(current_pos);
                    if team != target_team {
                        break;
                    }

                    if target_piece_type != 3 { 
                        // Can only castle with rooks.
                        break;
                    }

                    for piece in &board.pieces {
                        if piece.pos == current_pos && !piece.first_move {
                            break;
                        }
                    }
                    
                    actions.push(Action {
                        from: pos,
                        to: current_pos,
                        piece_type,
                        team,
                        capture: true,
                        info: CASTLING_MOVE
                    });
                    break;
                }
            }
        }
    }

    actions
}


pub struct KingPiece {
    deltas: Vec<i16>,
    sliders: Vec<i16>
}
impl KingPiece {
    pub fn new(row_gap: i16) -> Self {
        KingPiece {
            deltas: vec![
                1,
                -1,
                row_gap,
                -row_gap,
                row_gap + 1,
                row_gap - 1,
                -row_gap + 1,
                -row_gap - 1,
            ],
            sliders: vec![
                1,
                -1
            ]
        }
    }
}

impl Piece for KingPiece {
    fn get_actions(&self, board: &Board, piece_info: &PieceGenInfo) -> Vec<Action> {
        let mut actions = get_actions_delta(&self.deltas, board, piece_info);
        actions.extend(get_actions_castling(&self.sliders, board, piece_info));
        actions
    }

    fn can_control(&self, board: &Board, piece_info: &PieceGenInfo, targets: &Vec<i16>) -> bool {
        let mut can_control = false;
        for action in get_actions_delta(&self.deltas, board, piece_info) {
            if targets.contains(&action.to) {
                can_control = true;
                break;
            }
        }
        can_control
    }

    fn get_icon(&self) -> &str {
        "♚"
    }

    fn get_material_value(&self) -> i32 {
        1000
    }

    fn duplicate(&self) -> Box<dyn Piece> {
        Box::new(KingPiece {
            deltas: self.deltas.clone(),
            sliders: self.deltas.clone()
        })
    }

    fn make_move(&self, board: &mut Board, action: Action) {
        if action.info == NORMAL_MOVE {
            let old_pieces = board.pieces.clone();

            let MakeMoveResults {
                from_state,
                to_state,
            } = base_make_move(board, action);

            let past_move = StoredMove {
                action,
                from_previous: from_state,
                to_previous: to_state,
                pieces: old_pieces,
                state: None
            };

            board.history.push(past_move);
        } else if action.info == CASTLING_MOVE {
            let old_pieces = board.pieces.clone();
            let old_state = Some(board.state.clone());

            let castle_dir = (action.to - action.from).signum();
            let new_king_pos = action.from + (2 * castle_dir);
            let new_rook_pos = action.from + castle_dir;

            let from_usize = action.from as usize;
            let to_usize = action.to as usize;
        
            let from_state = board.state[from_usize];
            let to_state = board.state[to_usize];
            
            board.state[from_usize] = 1;
            board.state[to_usize] = 1;

            board.state[new_king_pos as usize] = from_state;
            board.state[new_rook_pos as usize] = to_state;

            let from_pos_all = board
                .pieces
                .iter()
                .position(|piece| piece.pos == action.from)
                .unwrap();

            let to_pos_all = board
                .pieces
                .iter()
                .position(|piece| piece.pos == action.to)
                .unwrap();

            board.pieces[from_pos_all].pos = new_king_pos;
            board.pieces[from_pos_all].first_move = false;
            board.pieces[to_pos_all].pos = new_rook_pos;
            board.pieces[to_pos_all].first_move = false;

            let past_move = StoredMove {
                action,
                from_previous: from_state,
                to_previous: to_state,
                pieces: old_pieces,
                state: old_state
            };

            board.history.push(past_move);
        }
    }
}
