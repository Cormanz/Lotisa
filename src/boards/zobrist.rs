use rand::prelude::*;

use crate::boards::Board;

pub fn generate_zobrist(pieces: i16, teams: i16, positions: i16) -> Vec<usize> {
    let mut rng = rand_hc::Hc128Rng::from_entropy();
    let len = (((pieces * teams) + 2) * 2) * positions;

    let mut zobrist: Vec<usize> = Vec::with_capacity(len as usize);
    for i in 0..len {
        zobrist.insert(i as usize, rng.next_u64() as usize);
    }

    zobrist
}

pub fn hash_board(board: &Board, moving_team: i16, zobrist: &Vec<usize>) -> usize {
    let mut hash: usize = 0;
    let mut ind: i16 = 0;
    let positions = board.row_gap * board.col_gap;
    hash ^= zobrist[moving_team as usize];
    for piece in &board.state {
        let piece = *piece;
        if piece == 0 { continue; }

        let first_move = board.pieces.iter()
            .find(|piece| piece.pos == ind)
            .map_or(false, |piece| piece.first_move);
        
        hash ^= zobrist[(ind + positions * if first_move { 1 } else { 0 } + (positions * 2) * piece) as usize];
        ind += 1;
    }

    return hash;
}