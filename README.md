# Lotisa

[![forthebadge](https://forthebadge.com/images/badges/made-with-rust.svg)](https://forthebadge.com)
[![forthebadge](https://forthebadge.com/images/badges/powered-by-oxygen.svg)](https://forthebadge.com)
[![forthebadge](https://forthebadge.com/images/badges/not-a-bug-a-feature.svg)](https://forthebadge.com)
[![forthebadge](https://forthebadge.com/images/badges/fuck-it-ship-it.svg)](https://forthebadge.com)
![have not released a version yet](https://badgen.net/badge/release/nothing%20yet!/red?)

## Overview

Lotisa is a chess movegen library and chess engine coded in Rust as part of the **Chesstastic** project. The goal of the engine is to reach a playing strength around at least 2,000 elo for the base chess game, and to be easily extendible to other variants. The engine is coded using to allow for following concepts:

- Custom Board Sizes _(up to roughly 180x180)_
- Custom Piece Types
- Custom Rules _(eg. castling through check)_
- More than 2 Teams

The engine uses a **10x12** board representation, where there's an **8x8** board inside of it, but additional squares are added to speed up the out of bounds check. Each piece is represented as an `i16` with the following formula: `piece_type + (PIECE_TYPES * team) + 2`, which allows for up to 16,384 piece types if there are two teams.

Chesstastic is a project to allow for players to fight against other players or chess bots with custom chess variants of their own choosing or creation, and to be able to analyze those very games. Lotisa is meant to allow for players to play against it on any variant, or to have it analyze games (and ideally, even explain moves.) It's also meant to validate chess moves in general for Chesstastic.

# Custom Board Sizes

Traditionally, chess engines use [Bitboards](https://www.chessprogramming.org/Bitboards) to represent the boards, where there are twelve different 64-bit integers for each piece and team, and the pieces are represented as a `1` if they exist on that bitboard, or a `0` if they don't. This allows for bitwise operators, which modern computers have already optimized into oblivion to be used to drastically increase the speed of chess move generation. Because Lotisa is meant to allow for custom board sizes, this cannot be taken advantage of _(there's the option of bitsets which would be slower, however.)_ Lotisa cannot represent the board using this model, however, because boards can be bigger than 8x8.

Lotisa represents boards as follows: `(cols + buffer) by (rows + 2 * buffer)`, or for a default chess board, `10 by 12`. We have a special variable called `buffer` which indicates how many squares should be added on each side of the board to account for out of bounds checks. By default, this is `2`, since the farthest any piece could reach into out of bounds would be `2` due to the Knight's movement. It's customizable, however, in case new types of pieces could move further. This is slower then Bitboards, but is still faster than using `cols by rows` because of the faster out of bounds check.

Lotisa allows for boards to be initialized using the following functions:

```rust
impl Board {
    pub fn new(piece_types: i16, buffer_amount: i16, teams: i16, (rows, cols): (i16, i16)) -> Board
    pub fn load_fen(fen: &str) -> Board
}
```
The `new` function allows for you to initialize an empty board _(with out of bounds squares and empty squares already separated.)_ You'll have to do the job of filling it yourself. `load_fen`, on the other hand, allows you to create a typical 8x8 board with a normal chess FEN. Here's code examples of both methods:
```rust
let board_new = Board::new(6, 2, 2, (8, 8), create_default_piece_map(10));
let board_fen = Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
```

# Custom Piece Types

Lotisa represents each piece type as an `i16`. One option was to use an enum for each piece type, but an `i16` is easier to manipulate with math on the board itself, and allows for consumers of the Lotisa library to add their own pieces.

Lotisa provides the following trait for implementing Piece behavior:

```rust
pub trait Piece {
    fn can_attack(&self, board: &Board, piece_info: &PieceGenInfo, target: i16) -> bool;
    fn get_actions(&self, board: &Board, piece_info: &PieceGenInfo) -> Vec<Action>;
    fn get_icon(&self) -> &str;
}
```

`can_attack` is used to check if a piece is threatening to _capture_ a particular square, which is used for optimizing checks.
`get_actions` provides all of the psuedolegal moves (all legal moves a piece can make, not accounting for king captures or checks) a piece can make.
`get_icon` provides the emoji-icon of the piece, which makes it easy to see the board's state with `board.print_board()`.

The default implementation of `can_attack` is provided by `Piece` itself, which just checks if the target is threatened by any of your moves. However, you are **strongly advised** to reimplement it if possible, as it will greatly speed up the engine's legal move generation (checking if moves put you in check.)

Lotisa stores a `piece_map` with every board (you may have spotted the `create_default_piece_map(10)` argument in `Board::new` earlier.) This piece map is a `FnvHashMap` that has `i16` keys of each piece type, and the `Piece` trait that describes how they move. If you choose to initialize your own board, you could inject your own piece like so:

```rust
let mut piece_map = create_default_piece_map(10);
piece_map.insert(6, KnookPiece::new(10));
let board = Board::new(6, 2, 2, (8, 8), piece_map);
```

# Custom Rules

This hasn't yet been implemented, but in the future, you'll be allowed to customize **additional move restrictions** that stop specific moves from happening. Perhaps you want to make it illegal to have your king away more than 1 square away from other piece, or perhaps you want to disable checks and allow for kings to be captured. Lotisa aims to make this possible.

# Performance

The performance as of right now is suboptimal, most likely due to my inexperience with Rust and my fierce battle to satisfy the borrow checker. If you spot anything that seems unoptimized, feel free to make a contribution!