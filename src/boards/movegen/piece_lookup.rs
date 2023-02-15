/*
    I am probably implementing this idea in a non-Rustic way, but this is the idea:
    Piece types are represented as an i16, and Piece is a special trait that defines how a piece can move
    The PieceMap defines each piece type and its respective trait implementation
    I assume the problem is that this incurs a runtime cost for looking up the piece's movements
    But, this gives the following implementation benefits:

    - 32,768 piece types (in practice this may be lower because i16 for piece index is shared with piece team, too)
    - Consumers of the library can implement a piece type num as its own piece trait

    Something like a match would probably be way more efficient but I have no idea how that would work without sacrificing dynamic piece types.

    Before the implementation of PieceMap, this was about 2.7M per second, now it's 1.1M per second.
*/

use fnv::FnvHashMap;

use super::{BishopPiece, KingPiece, KnightPiece, PawnPiece, Piece, QueenPiece, RookPiece};

struct PieceMapInfo {
    pawn: Box<dyn Piece>,
    knight: Box<dyn Piece>,
    bishop: Box<dyn Piece>,
    rook: Box<dyn Piece>,
    queen: Box<dyn Piece>,
    king: Box<dyn Piece>,
}

pub trait PieceLookup {
    fn lookup(&self, piece_type: i16) -> &Box<dyn Piece>;
}

pub type PieceMap = FnvHashMap<i16, Box<dyn Piece>>;

pub struct PieceMapLookup {
    pub map: PieceMap,
}

impl PieceMapLookup {
    pub fn new(map: PieceMap) -> PieceMapLookup {
        PieceMapLookup { map }
    }

    pub fn template(mut map: PieceMap, edit: Box<dyn Fn(&mut PieceMap) -> ()>) -> PieceMapLookup {
        edit(&mut map);
        PieceMapLookup { map }
    }

    pub fn default_map(row_gap: i16) -> PieceMap {
        let mut map: PieceMap = FnvHashMap::with_capacity_and_hasher(6, Default::default());
        map.insert(0, Box::new(PawnPiece) as Box<dyn Piece>);
        map.insert(1, Box::new(KnightPiece::new(row_gap)) as Box<dyn Piece>);
        map.insert(2, Box::new(BishopPiece::new(row_gap)) as Box<dyn Piece>);
        map.insert(3, Box::new(RookPiece::new(row_gap)) as Box<dyn Piece>);
        map.insert(4, Box::new(QueenPiece::new(row_gap)) as Box<dyn Piece>);
        map.insert(5, Box::new(KingPiece::new(row_gap)) as Box<dyn Piece>);
        map
    }

    pub fn default_template(
        row_gap: i16,
        edit: Box<dyn Fn(&mut PieceMap) -> ()>,
    ) -> PieceMapLookup {
        PieceMapLookup::template(PieceMapLookup::default_map(row_gap), edit)
    }
}

impl PieceLookup for PieceMapLookup {
    fn lookup(&self, piece_type: i16) -> &Box<dyn Piece> {
        self.map.get(&piece_type).unwrap()
    }
}

pub struct DefaultPieceLookup {
    info: PieceMapInfo,
}

impl DefaultPieceLookup {
    fn new(row_gap: i16) -> Self {
        DefaultPieceLookup {
            info: PieceMapInfo {
                pawn: Box::new(PawnPiece),
                knight: Box::new(KnightPiece::new(row_gap)),
                bishop: Box::new(BishopPiece::new(row_gap)),
                rook: Box::new(RookPiece::new(row_gap)),
                queen: Box::new(QueenPiece::new(row_gap)),
                king: Box::new(KingPiece::new(row_gap)),
            },
        }
    }
}

impl PieceLookup for DefaultPieceLookup {
    fn lookup(&self, piece_type: i16) -> &Box<dyn Piece> {
        return match piece_type {
            0 => &self.info.pawn,
            1 => &self.info.knight,
            2 => &self.info.bishop,
            3 => &self.info.rook,
            4 => &self.info.queen,
            5 => &self.info.king,
            _ => &self.info.pawn,
        };
    }
}

pub fn create_default_piece_lookup<'a>(row_gap: i16) -> Box<dyn PieceLookup> {
    Box::new(DefaultPieceLookup::new(row_gap)) as Box<dyn PieceLookup>
}
