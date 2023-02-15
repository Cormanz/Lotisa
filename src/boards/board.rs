use colored::{ColoredString, Colorize};
use fnv::FnvHashMap;

use super::{
    create_default_piece_lookup, generate_legal_moves, generate_moves, Piece, PieceLookup, PieceMap, PieceMapLookup,
};

//use super::Action;

pub type BoardState = Vec<i16>;

pub fn create_board_state(buffer_amount: i16, (rows, cols): (i16, i16)) -> BoardState {
    let mut state: BoardState = vec![];
    for row in 0..(rows + (2 * buffer_amount)) {
        for col in 0..(cols + buffer_amount) {
            state.push(
                if row < buffer_amount
                    || row >= (rows + buffer_amount)
                    || col < (buffer_amount / 2)
                    || col >= (cols + (buffer_amount / 2))
                {
                    0
                } else {
                    1
                },
            );
        }
    }

    return state;
}

pub struct StoredMove {
    pub action: Action,
    pub from_previous: i16,
    pub to_previous: i16,
    pub pieces: Vec<i16>,
}

#[derive(Debug)]
pub enum ActionType {
    MOVE,
    CAPTURE,
    FAIL,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Action {
    pub from: i16,
    pub to: i16,
    pub piece_type: i16,
    pub capture: bool,
    pub info: Option<i16>,
}

pub type PieceList = FnvHashMap<i16, Vec<i16>>;

#[derive(Copy, Clone)]
pub struct PieceInfo {
    pub pos: i16,
    pub piece_value: i16,
    pub team: i16,
    pub piece_type: i16,
}
pub struct Board {
    pub state: BoardState,
    pub pieces: Vec<i16>,
    pub reverse_pieces: FnvHashMap<i16, usize>,
    pub piece_types: i16,
    pub teams: i16,
    pub rows: i16,
    pub cols: i16,
    pub buffer_amount: i16,
    pub row_gap: i16,
    pub col_gap: i16,
    pub piece_lookup: Box<dyn PieceLookup>,
    pub history: Vec<StoredMove>
}

// TODO: Add reverse piece list to speed up removing items

impl Board {
    pub fn new(
        piece_types: i16,
        buffer_amount: i16,
        teams: i16,
        (rows, cols): (i16, i16),
        piece_lookup: Box<dyn PieceLookup>,
    ) -> Board {
        let state = create_board_state(buffer_amount, (rows, cols));

        return Board {
            state,
            reverse_pieces: FnvHashMap::with_capacity_and_hasher(32, Default::default()),
            pieces: Vec::with_capacity(32),
            piece_types,
            teams,
            rows,
            cols,
            buffer_amount,
            row_gap: rows + buffer_amount,
            col_gap: cols + (buffer_amount * 2),
            piece_lookup,
            history: Vec::with_capacity(500)
        };
    }

    pub fn display_board(&mut self) -> Vec<ColoredString> {
        let mut items: Vec<ColoredString> = vec![];

        let mut ind = 0;
        for row in self.state.chunks(self.row_gap as usize) {
            let all_empty = row.iter().all(|piece| *piece == 0);
            if all_empty {
                continue;
            };

            if ind != 0 {
                items.push("\n".white());
            }

            for col in row {
                let piece = *col;
                if piece == 0 {
                    continue;
                };
                if piece == 1 {
                    items.push("  ".white());
                    continue;
                }

                let team = self.get_team(piece);
                let piece_type = self.get_piece_type(piece, team);
                let piece_trait = &self.piece_lookup.lookup(piece_type);
                let piece_icon = piece_trait.get_icon();
                items.push(match team {
                    0 => piece_icon.white(),
                    1 => piece_icon.black(),
                    _ => "".red(),
                });
                items.push(" ".white());
            }

            ind += 1;
        }

        items
    }

    pub fn print_board(&mut self) {
        for el in self.display_board() {
            print!("{}", el);
        }
        println!("\n");
    }

    pub fn make_move(&mut self, action: Action) {
        let PieceInfo { 
            piece_type,
            ..
        } = self.get_piece_info(action.from);

        let piece_trait = self.piece_lookup.lookup(piece_type).duplicate();
        piece_trait.make_move(self, action);
    }

    pub fn undo_move(&mut self) {
        let undo = self.history.pop().unwrap();
        let StoredMove {
            action,
            to_previous,
            from_previous,
            pieces,
        } = undo;
        self.state[action.to as usize] = to_previous;
        self.state[action.from as usize] = from_previous;
        self.pieces = pieces;
    }

    /*
        Index 0 represents an out of bounds square and index 1 represents an empty square, so we add plus two to the index
    */
    pub fn get_piece_value(&self, piece_type: i16, team: i16) -> i16 {
        piece_type + (self.piece_types * team) + 2
    }

    pub fn get_team_min(&self, team: i16) -> i16 {
        (team * self.piece_types) + 2
    }

    pub fn get_team(&self, piece: i16) -> i16 {
        (piece - 2) / self.piece_types
    }

    pub fn get_piece_type(&self, piece: i16, team: i16) -> i16 {
        (piece - 2) - self.piece_types * team
    }

    pub fn get_piece_info(&self, pos: i16) -> PieceInfo {
        let piece_value = self.state[pos as usize];
        let team = self.get_team(piece_value);
        let piece_type = self.get_piece_type(piece_value, team);
        return PieceInfo {
            pos,
            piece_value,
            team,
            piece_type,
        };
    }

    pub fn get_row(&self, pos: i16) -> i16 {
        pos / self.row_gap
    }

    pub fn get_col(&self, pos: i16, row: i16) -> i16 {
        pos - (self.row_gap * row)
    }

    pub fn can_move(&self, pos: i16) -> bool {
        self.state[pos as usize] == 1
    }

    pub fn can_capture(&self, pos: i16, team: i16) -> bool {
        let state = self.state[pos as usize];
        state > 1 && self.get_team(state) != team
    }

    pub fn can_move_capture(&self, pos: i16, team: i16) -> ActionType {
        let state = self.state[pos as usize];
        match state {
            0 => ActionType::FAIL,
            1 => ActionType::MOVE,
            _ => {
                if self.get_team(state) != team {
                    ActionType::CAPTURE
                } else {
                    ActionType::FAIL
                }
            }
        }
    }

    pub fn can_control(&self, pos: i16, team: i16) -> ActionType {
        let state = self.state[pos as usize];
        match state {
            0 => ActionType::FAIL,
            1 => ActionType::MOVE,
            _ => ActionType::CAPTURE
        }
    }

    pub fn load_fen(fen: &str) -> Board {
        let fen_chunks = fen.split("/");
        let mut pieces: Vec<i16> = Vec::with_capacity(32);
        let mut reverse_pieces: FnvHashMap<i16, usize> =
            FnvHashMap::with_capacity_and_hasher(32, Default::default());
        let mut board = Board::new(6, 2, 2, (8, 8), create_default_piece_lookup(10));

        for (row_ind, chunk) in fen_chunks.enumerate() {
            let mut col_ind: usize = 0;
            for col in chunk.chars() {
                if col.is_numeric() {
                    let empty_spaces = col.to_digit(10).unwrap();
                    col_ind += empty_spaces as usize;
                    continue;
                }
                let team = if col.is_ascii_uppercase() { 0 } else { 1 };
                let piece_type = match col.to_ascii_lowercase() {
                    'p' => 0,
                    'n' => 1,
                    'b' => 2,
                    'r' => 3,
                    'q' => 4,
                    'k' => 5,
                    _ => 0,
                };
                let piece = board.get_piece_value(piece_type, team);
                let piece_pos = (col_ind + 1) + 10 * (row_ind + 2);
                board.state[piece_pos] = piece;
                col_ind += 1;

                let piece_pos_i16 = piece_pos as i16;

                reverse_pieces.entry(piece_pos_i16).or_insert(pieces.len());
                pieces.push(piece_pos_i16);
            }
        }

        board.pieces = pieces;

        board
    }

    pub fn generate_moves(&mut self, team: i16) -> Vec<Action> {
        generate_moves(self, team)
    }

    pub fn generate_legal_moves(&mut self, team: i16) -> Vec<Action> {
        generate_legal_moves(self, team)
    }
}
