use crate::boards::{Action, Board};

use super::in_check;

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

        GameResult::Ongoing
    }

    fn duplicate(&self) -> Box<dyn WinConditions> {
        Box::new(DefaultWinConditions)
    }
}