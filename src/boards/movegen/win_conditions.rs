use std::collections::HashSet;

use crate::boards::{Action, Board, hash_board};

use super::in_check;

pub fn is_draw_by_repetition(board: &mut Board) -> bool {
    let mut len = board.history.len();
    if len < 20 {
        len = 20;
    }
    let mut min_undos = len - 20;
    if min_undos > 100_000_000 {
        min_undos = 0;
    }
    let mut undos = board.history.as_slice()[min_undos..].to_vec();
    let mut hashes: Vec<usize> = Vec::with_capacity(undos.len());

    let mut ind = 0;
    for _ in &undos {
        if ind % 2 == 0 {
            hashes.push(hash_board(board, board.moving_team, &board.zobrist));
        }
        board.undo_move();
        ind += 1;
    }
    
    for undo in &undos {
        board.make_move(undo.action);
    }

    let mut hash_set: HashSet<usize> = HashSet::with_capacity(undos.len());
    for hash in hashes {
        if hash_set.contains(&hash) {
            return true;
        }

        hash_set.insert(hash);
    }

    return false;
}

pub enum GameResult {
    Win,
    Lose,
    Draw,
    Ongoing
}

pub trait WinConditions {
    fn compute(&self, board: &mut Board, actions: &Vec<Action>) -> GameResult;
    fn duplicate(&self) -> Box<dyn WinConditions>;
}

pub struct DefaultWinConditions;

impl WinConditions for DefaultWinConditions {
    fn compute(&self, board: &mut Board, actions: &Vec<Action>) -> GameResult {
        if actions.len() == 0 {
            if in_check(board, board.moving_team, board.row_gap) {
                return GameResult::Lose;
            }

            return GameResult::Draw;
        }

        if is_draw_by_repetition(board) {
            return GameResult::Draw;
        }

        GameResult::Ongoing
    }

    fn duplicate(&self) -> Box<dyn WinConditions> {
        Box::new(DefaultWinConditions)
    }
}