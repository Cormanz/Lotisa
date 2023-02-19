use crate::{boards::{Action, Board}, communication::{UCICommunicator, Communicator}};

use super::MAX_DEPTH;

pub struct PV {
	pub length: [ i16; MAX_DEPTH ],
	pub table: [ [ Option<Action>; MAX_DEPTH ]; MAX_DEPTH ]
}

impl PV {
    pub fn init_pv(&mut self, ply: i16) {
        self.length[ply as usize] = ply;
    }
    
    pub fn update_pv(&mut self, ply: i16, best_move: Option<Action>) {
        self.table[ply as usize][ply as usize] = best_move;
        for next_ply in (ply + 1)..(self.length[(ply as usize) + 1]) {
            self.table[ply as usize][next_ply as usize] = self.table[(ply + 1) as usize][next_ply as usize];
        }
        self.length[ply as usize] = self.length[(ply + 1) as usize];
    }
    
    pub fn display_pv(&mut self, uci: &mut UCICommunicator) -> String {
        let mut pv_actions: Vec<String> = Vec::with_capacity(self.table[0].len());
        let mut pv_table = self.table[0].clone();
        for action in &pv_table {
            if action.is_none() { break; }
            if let Some(action) = action {
                let from_pos_all = uci.board
                    .pieces
                    .iter()
                    .position(|piece| piece.pos == action.from);

                if from_pos_all.is_none() {
                    break;
                }

                pv_actions.push(uci.encode(action));
                uci.board.make_move(*action);
            }
        }
    
        for action in &self.table[0] {
            if let Some(_) = action {
                uci.board.undo_move();
            }
        }
    
        pv_actions.join(" ")
    }
}