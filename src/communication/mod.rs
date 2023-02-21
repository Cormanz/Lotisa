use crate::boards::{Action, Board};

pub trait Communicator {
    fn encode(&mut self, action: &Action) -> String;
    fn decode(&mut self, action: String) -> Action;
    fn encode_pos(&mut self, pos: i16) -> String;
    fn decode_pos(&mut self, pos: String) -> i16;
}

pub struct UCICommunicator {
    pub board: Board,
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
    fn encode(&mut self, action: &Action) -> String {
        let buffer_amount = self.board.buffer_amount;
        let to = if action.piece_type == 5 && action.info == 1 {
            action.from + ((action.to - action.from).signum() * 2)
        } else {
            action.to
        };

        return format!(
            "{}{}{}",
            encode_uci_pos(&self.board, action.from, buffer_amount),
            encode_uci_pos(&self.board, to, buffer_amount),
            if action.piece_type == 0 && action.info >= 0 {
                match action.info {
                    1 => "n",
                    2 => "b",
                    3 => "r",
                    4 => "q",
                    _ => "",
                }
            } else {
                ""
            }
        );
    }

    fn decode(&mut self, action: String) -> Action {
        let buffer_amount = self.board.buffer_amount;
        let from = decode_uci_pos(&self.board, &action[0..2], buffer_amount);
        let mut to = decode_uci_pos(&self.board, &action[2..4], buffer_amount);
        let piece_info = self.board.get_piece_info(from);

        let en_passant = piece_info.piece_type == 0
            && ((from - to).abs() % self.board.row_gap) != 0
            && self.board.state[to as usize] == 1;

        let castling = if piece_info.piece_type == 5 && (from - to).abs() == 2 {
            to = self
                .board
                .generate_moves()
                .iter()
                .find(|action| {
                    action.from == from
                        && action.info == 1
                        && (from - to).signum() == (action.from - action.to).signum()
                })
                .unwrap()
                .to;

            true
        } else {
            false
        };

        Action {
            from,
            to,
            piece_type: piece_info.piece_type,
            team: piece_info.team,
            capture: self.board.state[to as usize] > 1 && !en_passant,
            info: if piece_info.piece_type == 0 {
                if action.len() == 5 {
                    match action.chars().nth(4).unwrap() {
                        'n' => 1,
                        'b' => 2,
                        'r' => 3,
                        'q' => 4,
                        _ => 0,
                    }
                } else if en_passant {
                    -3
                } else if (from - to).abs() == 2 * self.board.row_gap {
                    -2
                } else {
                    -1
                }
            } else if piece_info.piece_type == 5 {
                if castling {
                    1
                } else {
                    0
                }
            } else {
                0
            },
        }
    }

    fn encode_pos(&mut self, pos: i16) -> String {
        return encode_uci_pos(&self.board, pos, self.board.buffer_amount);
    }

    fn decode_pos(&mut self, pos: String) -> i16 {
        return decode_uci_pos(&self.board, &pos, self.board.buffer_amount);
    }
}
