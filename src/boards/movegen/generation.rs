use std::collections::HashMap;

use fnv::FnvHashMap;

use crate::boards::{ActionType, Board, Action};

use super::piece_types::{Piece, PawnPiece, KnightPiece, QueenPiece, RookPiece, KingPiece, BishopPiece};

pub struct PieceGenInfo {
    pub pos: i16,
    pub team: i16,
    pub row_gap: i16,
    pub piece_type: i16
}

pub type PieceMap = FnvHashMap<i16, Box<dyn Piece>>;

/*
    I am probably implementing this idea in a non-Rustic way, but this is the idea:
    Piece types are represented as an i16, and Piece is a special trait that defines how a piece can move
    The PieceMap defines each piece type and its respective trait implementation
    I assume the problem is that this incurs a runtime cost for looking up the piece's movements
    But, this gives the following implementation benefits:

    - 32,768 piece types (in practice this may be lower because i16 for piece index is shared with piece team, too)
    - Consumers of the library can implement a piece type num as its own piece trait

    Something like a match would probably be way more efficient but I have no idea how that would work without sacrificing dynamic piece types.

    Before the implementation of PieceMap, this was about 2.7M per second, now it's 1.1M per second.
*/
pub fn create_default_piece_map(row_gap: i16) -> PieceMap {
    let mut map: PieceMap = FnvHashMap::with_capacity_and_hasher(6, Default::default());
    map.insert(0, Box::new(PawnPiece));
    map.insert(1, Box::new(KnightPiece::new(row_gap)));
    map.insert(2, Box::new(BishopPiece::new(row_gap)));
    map.insert(3, Box::new(RookPiece::new(row_gap)));
    map.insert(4, Box::new(QueenPiece::new(row_gap)));
    map.insert(5, Box::new(KingPiece::new(row_gap)));
    map
}

pub fn generate_moves(board: &Board, required_team: i16) -> Vec<Action> {
    let Board { state, row_gap, pieces, .. } = board;
    let mut actions: Vec<Action> = Vec::with_capacity(64);
    let row_gap = *row_gap;

    for pos in pieces {
        let pos = *pos;
        let piece = state[pos as usize];
        let team = board.get_team(piece);
        if team != required_team { continue; }

        let piece_type = board.get_piece_type(piece, team);
        let piece_info = PieceGenInfo {
            pos,
            row_gap,
            team,
            piece_type
        };
        actions.extend(board.piece_map[&piece_type].get_actions(board, &piece_info));
    }

    actions
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

        let undo = board.make_move(action);

        let king = board.get_piece_value(5, required_team);
        let king = *board.pieces.iter().find(|piece| board.state[**piece as usize] == king).unwrap();
        let mut can_add = true;
        for pos in &board.pieces {
            let pos = *pos;
            let pos_usize = pos as usize;
            let piece = board.state[pos_usize];
            let team = board.get_team(piece);
            if team == required_team { 
                continue; 
            }

            let piece_type = board.get_piece_type(piece, team);
            let piece_info = PieceGenInfo {
                pos,
                row_gap,
                team,
                piece_type
            };

            /*let piece_handler: Box<dyn Piece> = match piece_type {
                0 => Box::new(PawnPiece),
                1 => Box::new(KnightPiece::new(row_gap)),
                2 => Box::new(BishopPiece::new(row_gap)),
                3 => Box::new(RookPiece::new(row_gap)),
                4 => Box::new(QueenPiece::new(row_gap)),
                5 => Box::new(KingPiece::new(row_gap)),
                _ => Box::new(PawnPiece)
            };*/
            let piece_handler = board.piece_map.get(&piece_type).unwrap();
            if piece_handler.can_attack(board, &piece_info, king) {
                can_add = false;
                break;
            }
        }

        board.undo_move(undo);
        if can_add { new_actions.push(action); }
    }

    new_actions
}