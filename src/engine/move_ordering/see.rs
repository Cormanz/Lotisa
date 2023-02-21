use crate::boards::{Action, Board, PieceGenInfo, PieceInfo};

pub fn see(board: &mut Board, square: i16, moving_team: i16, current_attacker: Option<i16>) -> i32 {
    let row_gap = board.row_gap;
    let targets = vec![square];
    let pieces = board.pieces.clone();
    let attacking_pieces = pieces
        .iter()
        .filter(|piece| {
            let pos = piece.pos;
            let PieceInfo {
                piece_type, team, ..
            } = board.get_piece_info(pos);
            if team != moving_team {
                return false;
            }
            let piece_trait = board.piece_lookup.lookup(piece_type).duplicate();
            piece_trait.can_control(
                board,
                &PieceGenInfo {
                    pos,
                    team,
                    row_gap,
                    piece_type,
                },
                &targets,
            )
        })
        .collect::<Vec<_>>();
    if attacking_pieces.len() == 0 {
        return 0;
    }

    let attacker: i16 = if let Some(attacker) = current_attacker {
        attacker
    } else {
        let mut smallest_attacker: i16 = 0;
        let mut smallest_material: i32 = 2_000_000_000;
        for attacker in attacking_pieces {
            let piece = *attacker;
            let pos = piece.pos;
            let PieceInfo { piece_type, .. } = board.get_piece_info(pos);
            let piece_trait = board.piece_lookup.lookup(piece_type);
            let material = piece_trait.get_material_value();
            if material < smallest_material {
                smallest_attacker = pos;
                smallest_material = material;
            }
        }
        smallest_attacker
    };

    let attacker_type = board.get_piece_info(attacker).piece_type;

    let PieceInfo {
        piece_type: captured_type,
        team,
        ..
    } = board.get_piece_info(square);
    let square_value = board
        .piece_lookup
        .lookup(captured_type)
        .get_material_value();

    board.make_move(Action {
        from: attacker,
        to: square,
        capture: true,
        piece_type: attacker_type,
        team,
        info: match attacker_type {
            0 => -1,
            _ => 0,
        },
    });

    let value = square_value - see(board, square, board.get_next_team(moving_team), None);
    board.undo_move();
    return value;
}
