use std::collections::HashMap;

use fnv::FnvHashMap;

use crate::boards::{Action, ActionType, Board, PersistentPieceInfo};

use super::piece_types::{
    BishopPiece, KingPiece, KnightPiece, PawnPiece, Piece, QueenPiece, RookPiece,
};

pub struct PieceGenInfo {
    pub pos: i16,
    pub team: i16,
    pub row_gap: i16,
    pub piece_type: i16,
}

pub fn generate_moves(board: &mut Board, required_team: i16) -> Vec<Action> {
    let mut actions: Vec<Action> = Vec::with_capacity(64);
    let row_gap = board.row_gap;

    for PersistentPieceInfo { pos, .. } in &board.pieces {
        let pos = *pos;
        let piece = board.state[pos as usize];
        let team = board.get_team(piece);
        if team != required_team {
            continue;
        }

        let piece_type = board.get_piece_type(piece, team);
        let piece_info = PieceGenInfo {
            pos,
            row_gap,
            team,
            piece_type,
        };
        let piece_trait = board.piece_lookup.lookup(piece_type).duplicate();
        actions.extend(piece_trait.get_actions(board, &piece_info));
    }

    actions
}

pub fn in_check(board: &mut Board, moving_team: i16, row_gap: i16) -> bool {
    let king = board.get_piece_value(5, moving_team);
    let king = board
        .pieces
        .iter()
        .find(|piece| board.state[piece.pos as usize] == king)
        .map(|piece| piece.pos)
        .unwrap();
    let king_vec = vec![king];
    for PersistentPieceInfo { pos, .. } in &board.pieces {
        let pos = *pos;
        let pos_usize = pos as usize;
        let piece = board.state[pos_usize];
        let team = board.get_team(piece);
        if team == moving_team {
            continue;
        }

        let piece_type = board.get_piece_type(piece, team);
        let piece_info = PieceGenInfo {
            pos,
            row_gap,
            team,
            piece_type,
        };

        let piece_handler = board.piece_lookup.lookup(piece_type).duplicate();
        if piece_handler.can_control(board, &piece_info, &king_vec) {
            return true;
        }
    }

    false
}

pub fn generate_legal_moves(board: &mut Board, required_team: i16) -> Vec<Action> {
    let Board { row_gap, .. } = board;
    let row_gap = *row_gap;

    let actions = generate_moves(board, required_team);
    let mut new_actions: Vec<Action> = vec![];

    for action in actions {
        if action.capture {
            let target_value = board.state[action.to as usize];
            let target_team = board.get_team(target_value);
            if board.get_piece_type(target_value, target_team) == 5 {
                continue;
            }
        }

        board.make_move(action);
        let can_add = !in_check(board, required_team, row_gap);
        board.undo_move();

        if can_add {
            new_actions.push(action);
        }
    }

    new_actions
}
