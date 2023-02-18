use colored::{ColoredString, Colorize};
use fnv::FnvHashMap;

use crate::communication::{UCICommunicator, Communicator};

use super::{
    create_default_piece_lookup, generate_legal_moves, generate_moves, Piece, PieceLookup,
    PieceMap, PieceMapLookup, WinConditions, DefaultWinConditions, Restrictor, DefaultRestrictor,
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

#[derive(Debug)]
pub struct StoredMove {
    pub action: Action,
    pub from_previous: i16,
    pub to_previous: i16,
    pub pieces: Vec<PersistentPieceInfo>,
    pub state: Option<Vec<i16>>,
}

impl StoredMove {
    pub fn duplicate(&self) -> Self {
        StoredMove {
            action: self.action.clone(),
            from_previous: self.from_previous,
            to_previous: self.to_previous,
            pieces: self.pieces.clone(),
            state: None,
        }
    }
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
    pub team: i16,
    pub piece_type: i16,
    pub capture: bool,
    pub info: i16,
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
    pub pieces: Vec<PersistentPieceInfo>,
    pub reverse_pieces: FnvHashMap<i16, usize>,
    pub piece_types: i16,
    pub teams: i16,
    pub rows: i16,
    pub cols: i16,
    pub buffer_amount: i16,
    pub row_gap: i16,
    pub col_gap: i16,
    pub moving_team: i16,
    pub piece_lookup: Box<dyn PieceLookup>,
    pub win_conditions: Box<dyn WinConditions>,
    pub restrictors: Vec<Box<dyn Restrictor>>,
    pub history: Vec<StoredMove>
}

#[derive(Clone, Copy, Debug)]
pub struct PersistentPieceInfo {
    pub pos: i16,

    /*
        A lot of chess engines don't use this "first_move" property. There are two cases where Lotisa uses it:

        - Double Pawn Moves (TBD)
        - Castling

        Instead, for these two, they do the following:

        - Double Pawn Moves: Check if the pawn is at the starting row (because pawns can't move back)
        - Castling: Store two bools/bits for whether the king can castle kingside or queenside

        Lotisa doesn't do these, for the sake of extendibility and simplicity.
        For instance, Double Pawn Moves may be trickier to implement on specific board setups where pawns aren't all on one row.
        Or if a consumer wants to implement new rules based on first moves, this makes it way easier.
        If we didn't have this, consumers would have to find some sort of hacky-way to detect first moves with move generation, and it would not be fun.

        As for FEN parsing, "first_move" will be inferred base on information regarding castling rights or pawn placement, so compatibility will be affirmed there.
    */
    pub first_move: bool,
}

// TODO: Add reverse piece list to speed up removing items

impl Board {
    pub fn new(
        piece_types: i16,
        buffer_amount: i16,
        teams: i16,
        (rows, cols): (i16, i16),
        piece_lookup: Box<dyn PieceLookup>,
        win_conditions: Box<dyn WinConditions>,
        restrictors: Vec<Box<dyn Restrictor>>
    ) -> Board {
        let state = create_board_state(buffer_amount, (rows, cols));

        return Board {
            state,
            reverse_pieces: FnvHashMap::with_capacity_and_hasher(32, Default::default()),
            pieces: Vec::with_capacity(32),
            piece_types,
            win_conditions,
            restrictors,
            teams,
            rows,
            cols,
            buffer_amount,
            moving_team: 0,
            row_gap: rows + buffer_amount,
            col_gap: cols + (buffer_amount * 2),
            piece_lookup,
            history: Vec::with_capacity(500),
        };
    }

    pub fn get_next_team(&self, team: i16) -> i16 {
        let team = team + 1;
        if team >= self.teams {
            0
        } else {
            team
        }
    }

    pub fn get_previous_team(&self, team: i16) -> i16 {
        let team = team - 1;
        if team >= self.teams {
            0
        } else {
            team
        }
    }
    
    pub fn next_team(&self) -> i16 {
        self.get_next_team(self.moving_team)
    }
    
    pub fn previous_team(&self) -> i16 {
        self.get_next_team(self.moving_team)
    }

    pub fn display_board(&self) -> Vec<ColoredString> {
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
                let piece_trait = self.piece_lookup.lookup(piece_type).duplicate();
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

    pub fn print_board(&self) {
        for el in self.display_board() {
            print!("{}", el);
        }
        println!("\n");
    }

    pub fn make_move(&mut self, action: Action) {
        let PieceInfo { piece_type, .. } = self.get_piece_info(action.from);

        let piece_trait = self.piece_lookup.lookup(piece_type).duplicate();
        piece_trait.make_move(self, action);
        self.moving_team = self.next_team();
    }

    pub fn undo_move(&mut self) -> StoredMove {
        let undo = self.history.pop().unwrap();
        let piece_trait = self.piece_lookup.lookup(undo.action.piece_type).duplicate();
        piece_trait.undo_move(self, &undo);
        self.moving_team = self.previous_team();
        undo
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
            _ => ActionType::CAPTURE,
        }
    }

    pub fn load_uci_pgn(uci_pgn: &str) -> UCICommunicator {
        let mut uci = Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w kqKQ -");

        for action in uci_pgn.split(" ") {
            if action.chars().nth(0).unwrap().is_numeric() {
                continue;
            }

            let action = uci.decode(action.to_string());
            uci.board.make_move(action);
        }

        uci
    }

    pub fn load_fen(fen: &str) -> UCICommunicator {
        let fen_parts = fen.split(" ").collect::<Vec<_>>();

        let mut uci = UCICommunicator {
            board: Board::load_fen_pieces(fen_parts[0])
        };

        let castling = fen_parts[2].chars().collect::<Vec<_>>();

        // TODO: For each type of castling, change "first_move"s

        if castling[0] != '-' {
            for (castling_type, pos) in [('k', 98), ('q', 91), ('K', 28), ('Q', 21)] {
                if !castling.contains(&castling_type) {
                    let pieces_position = uci.board.pieces.iter()
                        .position(|piece| piece.pos == pos);
                    if let Some(pieces_position) = pieces_position {
                        uci.board.pieces[pieces_position].first_move = false;
                    }
                }
            }
        }

        if fen_parts[3] != "-" {
            let pos = uci.decode_pos(fen_parts[3].to_string());
            let row_gap = uci.board.row_gap;
            let PieceInfo { team, .. } = uci.board.get_piece_info(pos);

            let from = match team {
                0 => pos + row_gap * 2,
                1 => pos - row_gap * 2,
                _ => pos
            };

            let action = Action {
                from,
                to: pos,
                piece_type: 0,
                team,
                capture: false,
                info: -2
            };

            let mut old_pieces = uci.board.pieces.clone();
            let to_index = uci.board.pieces.iter()
                .position(|piece| piece.pos == pos)
                .unwrap();

            old_pieces[to_index].pos = from;
            old_pieces[to_index].first_move = false;

            uci.board.history.push(StoredMove {
                action,
                from_previous: 2,
                to_previous: 1,
                pieces: old_pieces,
                state: None
            })
        }

        // TODO: Add last move to history for en passant

        uci.board.moving_team = match fen_parts[1] {
            "w" => 0,
            "b" => 1,
            _ => 0
        };

        uci
    }

    pub fn load_fen_pieces(fen: &str) -> Board {
        let fen_chunks = fen.split("/");
        let mut pieces: Vec<PersistentPieceInfo> = Vec::with_capacity(32);
        let mut board = Board::new(
            6, 2, 2, (8, 8), 
            create_default_piece_lookup(10), 
            Box::new(DefaultWinConditions),
            vec![ Box::new(DefaultRestrictor) ]
        );

        let min_row = board.buffer_amount;
        let max_row = board.rows + board.buffer_amount;

        let pawn_min_row = min_row + 1;
        let pawn_max_row = max_row - 2;

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

                pieces.push(PersistentPieceInfo {
                    pos: piece_pos_i16,
                    first_move: if piece_type == 0 {
                        let row = board.get_row(piece_pos_i16);
                        match team {
                            0 => row == pawn_max_row,
                            1 => row == pawn_min_row,
                            _ => false,
                        }
                    } else {
                        true
                    },
                });
            }
        }

        board.pieces = pieces;

        board
    }

    pub fn generate_moves(&mut self) -> Vec<Action> {
        generate_moves(self, self.moving_team)
    }

    pub fn generate_legal_moves(&mut self) -> Vec<Action> {
        generate_legal_moves(self, self.moving_team)
    }
}
