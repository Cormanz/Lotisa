use rand::prelude::*;

use crate::boards::Board;

pub fn generate_zobrist(pieces: i16, teams: i16, positions: i16) -> Vec<usize> {
    let mut rng = rand_hc::Hc128Rng::from_entropy();
    let len = (positions * ((pieces * teams) + teams) + 2) as usize;
    let mut zobrist: Vec<usize> = Vec::with_capacity(len);
    for i in 0..len {
        zobrist.insert(i, rng.next_u64() as usize);
    }

    zobrist
}

pub fn hash_board(board: &Board, moving_team: i16, zobrist: &Vec<usize>) -> usize {
    let mut hash: usize = 0;
    let mut ind: usize = 0;
    let positions = board.rows * board.cols;
    hash ^= zobrist[moving_team as usize];
    for piece in &board.state {
        let piece = *piece;
        if piece == 0 { continue; }
        hash ^= zobrist[ind + 2 + (positions * piece) as usize];
        ind += 1;
    }

    return hash;
}
