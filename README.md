# Lotisa

[![forthebadge](https://forthebadge.com/images/badges/made-with-rust.svg)](https://forthebadge.com)
[![forthebadge](https://forthebadge.com/images/badges/powered-by-oxygen.svg)](https://forthebadge.com)
[![forthebadge](https://forthebadge.com/images/badges/not-a-bug-a-feature.svg)](https://forthebadge.com)
[![forthebadge](https://forthebadge.com/images/badges/fuck-it-ship-it.svg)](https://forthebadge.com)
![have not released a version yet](https://badgen.net/badge/release/nothing%20yet!/red?)

# Overview

Lotisa is a chess movegen library and chess engine coded in Rust as part of the **Chesstastic** project. The goal of the engine is to reach a playing strength around at least 2,000 elo for the base chess game, and to be easily extendible to other variants.

Chesstastic is a project to allow for players to fight against other players or chess bots with custom chess variants of their own choosing or creation, and to be able to analyze those very games. Lotisa is meant to allow for players to play against it on any variant, or to have it analyze games (and ideally, even explain moves.) It's also meant to validate chess moves in general for Chesstastic.

# Engine

The Lotisa engine uses a **10x12** board representation, where there's an **8x8** board inside of it, but additional squares are added to speed up the out of bounds check. Each piece is represented as an `i16` with the following formula: `piece_type + (PIECE_TYPES * team) + 2`, which allows for up to 16,384 piece types if there are two teams.

## Search

- [Iterative Deepening](https://www.chessprogramming.org/Iterative_Deepening)
- [Quiescience Search](https://www.chessprogramming.org/Quiescence_Search)
- [Alpha Beta Search](https://www.chessprogramming.org/Alpha-Betav)
    - [Pruning](https://www.chessprogramming.org/Pruning)
        - [Transposition Table Lookup](https://www.chessprogramming.org/Transposition_Table)
        - [Delta Pruning](https://www.chessprogramming.org/Delta_Pruning)
        - [Null Move Pruning](https://www.chessprogramming.org/Null_Move_Pruning)
        - [Futility Pruning](https://www.chessprogramming.org/Futility_Pruning)
            - Extended Futility Pruning
        - [Quiescence SEE Pruning](https://www.chessprogramming.org/Static_Exchange_Evaluation)
    - [Reductions]
        - [Late Move Reductions](https://www.chessprogramming.org/Late_Move_Reductions)
    - [Move Ordering](https://www.chessprogramming.org/Move_Ordering)
        - [Move from Transposition Table](https://www.chessprogramming.org/Transposition_Table)
        - [Internal Iterative Deepening](https://www.chessprogramming.org/Internal_Iterative_Deepening)
        - [Static Exchange Evaluation](https://www.chessprogramming.org/Static_Exchange_Evaluation)
        - [Killer Heuristic](https://www.chessprogramming.org/Killer_Heuristic)
        - [Countermove Heuristic](https://www.chessprogramming.org/Countermove_Heuristic)
        - [History Heuristic](https://www.chessprogramming.org/History_Heuristic)

## Evaluation

- [Hand Crafted Evaluation](https://www.chessprogramming.org/Evaluation)
    - [Material Imbalance](https://www.chessprogramming.org/Material)
    - [Center Control](https://www.chessprogramming.org/Center_Control)
    - [Mobility](https://www.chessprogramming.org/Mobility)

## Performance Bottlenecks

The performance right now is held back by my move generation algorithm. In particular, converting **psuedolegal moves** into **legal moves**. These are the results of a benchmark I did on the **starting position** _(that is still on the main branch as of now)_:

| Stage |  Depth  |  Nodes Found  |  MS Taken  |  Nodes / S  |
|---|---|---|---|---|
| Psuedolegal Movegen  | 6  |  35,408,726 | 955ms | 35,766,000 |
| Legal Movegen  |  5  |  1,413,803  |  825ms  |  1,713,000  |
| Negamax (AB Pruning)  |  7  |  122,048  |  390ms  |  312,000  |

\*: Move Generation still lacks some chess features (en passant, castling, promotion), so the number of nodes found will be inaccurate.

\**: Negamax only uses an evaluation of *material imbalance*, *center control* and *mobility*.

\***: Nodes per second is done by multiplying the nodes per millisecond value by 1000.

I am currently working on optimizing performance, but the legal movegen perft is abysmally low. This will make it difficult to do many comprehensive optimizations that normal chess engines do.

The main bottleneck is from Psuedolegal to Legal, which is a **18x** decrease in nodes per second (where Legal to Negamax is only about 3x.) This is an incredibly low speed. From the testing I've done, I believe there are two major causes of this:

- Retrieving the piece trait information from `board.piece_maps`.
- Having to run `can_control`.

Until then, I'll keep working on the engine and hope hope I'll be able to optimize my move generation, or that someone would be willing to help me who's much more knowledgeable on Rust and/or Chess Programming.


# Extendibility

The Lotisa engine is meant to be as extendible as possible to other variants. It's coded to allow for the following concepts:

- Custom Board Sizes _(up to roughly 180x180)_
- Custom Piece Types
- Custom Rules _(eg. castling through check)_
- More than 2 Teams

## Custom Board Sizes

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
let board_new = Board::new(6, 2, 2, (8, 8), create_default_piece_lookup(10));
let board_fen = Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
```

## Custom Piece Types

Lotisa represents each piece type as an `i16`. One option was to use an enum for each piece type, but an `i16` is easier to manipulate with math on the board itself, and allows for consumers of the Lotisa library to add their own pieces.

Lotisa provides the following trait for implementing Piece behavior:

```rust
pub trait Piece {
    fn can_control(&self, board: &Board, piece_info: &PieceGenInfo, target: i16) -> bool;
    fn get_actions(&self, board: &Board, piece_info: &PieceGenInfo) -> Vec<Action>;
    fn get_icon(&self) -> &str;
}
```

`can_control` is used to check if a piece is threatening to _capture_ a particular square, which is used for optimizing checks.
`get_actions` provides all of the psuedolegal moves (all legal moves a piece can make, not accounting for king captures or checks) a piece can make.
`get_icon` provides the emoji-icon of the piece, which makes it easy to see the board's state with `board.print_board()`.

The default implementation of `can_control` is provided by `Piece` itself, which just checks if the target is threatened by any of your moves. However, you are **strongly advised** to reimplement it if possible, as it will greatly speed up the engine's legal move generation (checking if moves put you in check.)

Lotisa stores a `piece_lookup` with every board (you may have spotted the `create_default_piece_lookup` argument in `Board::new` earlier.) This piece map is a `PieceLookup` with the following implementation:

```rust
pub trait PieceLookup {
    fn lookup(&self, piece_type: i16) -> &Box<dyn Piece>;
}
```

You can implement your own `PieceLookup` with custom pieces as follows:

```rust
struct NewPieceLookup
impl PieceLookup for NewPieceLookup {
    fn lookup(&self, piece_type: i16) -> &Box<dyn Piece> {
        return match piece_type { 
            ...,
            6 => &Box::new(KnookPiece::new(8))
        }
    }
}

let board = Board::new(6, 2, 2, (8, 8), NewPieceLookup);
```

`PieceLookup` is defined as a trait for ease of use in-case users would like to implement their own piece lookup styles or optimizations. However, Lotisa makes the very specific and common use of _adding new pieces to the base chess game_ incredibly easy to implement using Piece Maps. Here's an example:

```rust
let board = Board::new(6, 2, 2, (8, 8), PieceMapLookup.default_template(8, |map| {
    map.insert(6, KnookPiece::new(8));
}));
```

Since FnvHashMap has a minor runtime cost, this would be slower than the default piece lookup of using a match statement, but it's incredibly developer friendly if you want to try it out.

## Custom Rules

This hasn't yet been implemented, but in the future, you'll be allowed to customize **additional move restrictions** that stop specific moves from happening. Perhaps you want to make it illegal to have your king away more than 1 square away from other piece, or perhaps you want to disable checks and allow for kings to be captured. Lotisa aims to make this possible.