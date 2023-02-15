use crate::boards::{Action, Board};

pub trait Communicator {
    fn encode(&self, action: &Action) -> String;
    fn decode(&self, action: String) -> Action;
}

pub struct UCICommunicator {
    pub board: Board
}

fn encode_uci_pos(board: &Board, pos: i16, buffer_amount: i16) -> String {
    let true_row = board.get_row(pos);
    let true_col = board.get_col(pos, true_row);
    let row = board.rows - (true_row - buffer_amount) - 1;
    let col = true_col - (buffer_amount / 2);
    let mut abcs = "abcdefghijklmnopqrstuvwxyz".chars();
    return format!("{}{}", abcs.nth(col as usize).unwrap(), row + 1);
}

fn decode_uci_pos(board: &Board, pos: &str, buffer_amount: i16) -> i16 {
    let mut abcs = "abcdefghijklmnopqrstuvwxyz".chars();
    let col_char = pos.chars().nth(0).unwrap();
    let mut col = abcs.position(|char| char == col_char).unwrap() as i16;
    let mut row = board.rows - (pos.chars().nth(1).unwrap().to_digit(10).unwrap() as i16);
    col += buffer_amount / 2;
    row += buffer_amount;
    return (row * board.row_gap) + col;
}


impl Communicator for UCICommunicator {
    fn encode(&self, action: &Action) -> String {
        let buffer_amount = self.board.buffer_amount;
        return format!("{}{}", 
            encode_uci_pos(&self.board, action.from, buffer_amount),
            encode_uci_pos(&self.board, action.to, buffer_amount)
        );
    }

    fn decode(&self, action: String) -> Action {
        let buffer_amount = self.board.buffer_amount;
        let from = decode_uci_pos(&self.board, &action[0..2], buffer_amount);
        let to = decode_uci_pos(&self.board, &action[2..4], buffer_amount);
        Action {
            from,
            to,
            piece_type: self.board.get_piece_info(from).piece_type,
            capture: self.board.state[to as usize] > 1,
            info: None
        }
    }
}