use std::collections::HashSet;

use fnv::FnvHashMap;
use rand::Rng;

use crate::boards::{generate_legal_moves, generate_moves, Action, Board, PieceGenInfo, PieceInfo};

const INNER_CENTER_SQUARES: [i16; 4] = [54, 55, 64, 65];

const CENTER_SQUARES: [i16; 16] = [
    43, 44, 45, 46, 53, 54, 55, 56, 63, 64, 65, 66, 73, 74, 75, 76,
];

const CENTER_BOX: [i16; 8] = [
    53, 54, 55, 56, 63, 64, 65, 66,
];

pub fn weigh_mobility_move(board: &mut Board, action: &Action) -> i32 {
    let material_value = board
        .piece_lookup
        .lookup(action.piece_type)
        .get_material_value();
    let mut score = 5;

    if CENTER_SQUARES.contains(&action.to) {
        score += 2;
    }

    score *= 1 + ((9000 - material_value) / 8000);

    if action.capture {
        let attacker_material = board
            .piece_lookup
            .lookup(action.piece_type)
            .get_material_value();
        let victim_piece_type = board.get_piece_info(action.to).piece_type;
        let victim_material = board
            .piece_lookup
            .lookup(victim_piece_type)
            .get_material_value();
        if victim_material > attacker_material {
            score += 30;
        }
    }

    score
}

pub struct MobilityInfo {
    piece_material: i32,
    count: i16
}

fn weigh_mobility_moves(board: &mut Board, actions: &Vec<Action>, opposing_actions: &Vec<Action>) -> i32 {
    let mut score: i32 = 0;
    let mut map: FnvHashMap<i16, MobilityInfo> = FnvHashMap::with_capacity_and_hasher(16, Default::default());

    let opposing_targets = opposing_actions.iter().map(|action| action.to).collect::<HashSet<_>>();

    for action in actions {
        let mut bonus = 2;
        let contested = opposing_targets.contains(&action.to);

        if !map.contains_key(&action.from) {
            map.insert(action.from, MobilityInfo {
                piece_material: board.piece_lookup.lookup(action.piece_type).get_material_value(),
                count: 0
            });
        }

        let targeting_white_zone = action.to >= 61;
        let inside_white_zone = action.from >= 61;
        let space_control = (action.team == 0 && !targeting_white_zone && inside_white_zone) || (action.team == 1 && targeting_white_zone && !inside_white_zone);
        let center_control = INNER_CENTER_SQUARES.contains(&action.from);
        
        if space_control && center_control {
            bonus += 4;
        } else if space_control || center_control {
            bonus += 1;
        } else if contested {
            bonus -= 1;
        }
        
        map.get_mut(&action.from).unwrap().count += bonus;
    }

    for (_, info) in map {
        let material_weight = (9000.0 - (info.piece_material as f64)) / 8000.0 * (3.0 / 4.0);
        let gain = (10.0 * ((info.count as f64).sqrt() * (0.25 + material_weight))) as i32;
        score += gain;
    }

    score
}

pub fn evaluate(board: &mut Board, pov_team: i16) -> i32 {
    let mut score: i32 = 0;
    let row_gap = board.row_gap;

    for piece in board.pieces.clone() {
        let PieceInfo {
            piece_type, team, ..
        } = board.get_piece_info(piece.pos);
        let team_multiplier = if team == pov_team { 1 } else { -1 };

        let piece_trait = board.piece_lookup.lookup(piece_type);
        let material_value = piece_trait.get_material_value();
        score += team_multiplier * material_value;

        if piece_type == 5 {
            let deltas = [
                1,
                -1,
                row_gap,
                -row_gap,
                row_gap + 1,
                row_gap - 1,
                -row_gap + 1,
                -row_gap - 1,
            ];

            let mut opposing_pieces: Vec<PieceGenInfo> = Vec::with_capacity(16);
            for sub_piece_info in &board.pieces {
                let sub_piece = board.state[sub_piece_info.pos as usize];
                let sub_team = board.get_team(sub_piece);
                if sub_team == team {
                    continue;
                }
                let sub_piece_type = board.get_piece_type(sub_piece, sub_team);
                opposing_pieces.push(PieceGenInfo {
                    pos: sub_piece_info.pos,
                    team: sub_team,
                    row_gap,
                    piece_type: sub_piece_type,
                });
            }

            let mut open_squares = 0;
            let mut empty_squares = 0;

            for delta in deltas {
                let new_pos = piece.pos + delta;
                let state = board.state[new_pos as usize];
                match state {
                    1 => {
                        empty_squares += 1;
                        open_squares += 1;
                        for sub_piece in &opposing_pieces {
                            let sub_piece_trait =
                                board.piece_lookup.lookup(sub_piece.piece_type).duplicate();
                            if sub_piece_trait.can_control(board, &sub_piece, &vec![new_pos]) {
                                open_squares -= 1;
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            }

            if empty_squares > 0 {
                let blocked_squares: i32 = empty_squares - open_squares;
                score -= 2_000
                    * ((blocked_squares * blocked_squares) / (empty_squares * empty_squares))
                    * team_multiplier;
            }
        }
    }

    let moves = generate_moves(board, pov_team);
    let opposing_moves = generate_moves(board, board.get_next_team(pov_team));
    score += weigh_mobility_moves(board, &moves, &opposing_moves);
    score -= weigh_mobility_moves(board, &opposing_moves, &moves);

    score
}
