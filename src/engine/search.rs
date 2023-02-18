use crate::boards::{Board, Action};
use super::{MIN_VALUE, evaluate, EvaluationScore};

pub fn search(board: &mut Board, mut alpha: i32, beta: i32, depth: i16, ply: i16, starting_team: i16) -> EvaluationScore {
   if depth == 0 {
      return EvaluationScore {
         score: evaluate(board, starting_team),
         best_move: None
      };
   }

   let actions = board.generate_legal_moves();
   let mut best_eval: Option<EvaluationScore> = None;
   for action in actions  {
      board.make_move(action);
      let eval = search(board, -beta, -alpha, depth - 1, ply + 1, starting_team);
      let score = -eval.score;
      board.undo_move();
      let eval = EvaluationScore {
         score,
         best_move: Some(action)
      };
      if score >= beta {
         return eval;
      }
      
      if score > alpha {
         alpha = score;
      }

      let mut new_best_eval = best_eval.is_none();
      if let Some(best_eval) = &best_eval {
         if score > best_eval.score {
            new_best_eval = true;
         }
      }

      if new_best_eval {
         best_eval = Some(eval);
      }
   }
   return best_eval.unwrap();
 }