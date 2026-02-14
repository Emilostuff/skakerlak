use shakmaty::{zobrist::Zobrist64, Move, Role, Square};

use crate::search::transposition::Bound;

const ZOBRIST_MASK: u64 = 0xFFFF_FFFF_FFFF_0000;
const ZOBRIST_OFFSET: u8 = 64;

const DEPTH_OFFSET: u8 = ZOBRIST_OFFSET;
const SCORE_OFFSET: u8 = 32;
const BOUND_OFFSET: u8 = 30;

/// Layout of a TTEntry.
/// | Zobrist upper 48 bits | depth 8 bits | score 32 bits |
pub type PackedRep = u128;

pub fn pack(zobrist: Zobrist64, mv: Move, score: i32, depth: u8, bound: Bound) -> PackedRep {
    let mut entry = 0;

    set_zobrist(&mut entry, zobrist);
    set_depth(&mut entry, depth);
    set_score(&mut entry, score);
    set_bound(&mut entry, bound);
    set_move(&mut entry, mv);

    entry
}

#[inline(always)]
fn set_zobrist(entry: &mut PackedRep, zobrist: Zobrist64) {
    *entry |= ((zobrist.0 & ZOBRIST_MASK) as u128) << ZOBRIST_OFFSET;
}

#[inline(always)]
pub fn matches_zobrist(entry: PackedRep, zobrist: Zobrist64) -> bool {
    let a_zobrist = ((entry >> ZOBRIST_OFFSET) as u64) & ZOBRIST_MASK;
    let b_zobrist = zobrist.0 & ZOBRIST_MASK;
    a_zobrist == b_zobrist
}

#[inline(always)]
fn set_depth(entry: &mut PackedRep, depth: u8) {
    *entry |= (depth as u128) << DEPTH_OFFSET;
}

#[inline(always)]
pub fn get_depth(entry: PackedRep) -> u8 {
    (entry >> DEPTH_OFFSET) as u8
}

#[inline(always)]
fn set_score(entry: &mut PackedRep, score: i32) {
    *entry |= (score as u128 & 0xFFFF_FFFF) << SCORE_OFFSET;
}

#[inline(always)]
pub fn get_score(entry: PackedRep) -> i32 {
    (entry >> SCORE_OFFSET) as i32
}

#[inline(always)]
fn set_bound(entry: &mut PackedRep, bound: Bound) {
    *entry |= (bound as u128) << BOUND_OFFSET;
}

#[inline(always)]
pub fn get_bound(entry: PackedRep) -> Bound {
    match (entry >> BOUND_OFFSET & 0b11) as u8 {
        0 => Bound::Exact,
        1 => Bound::Lower,
        2 => Bound::Upper,
        _ => unreachable!(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MoveType {
    Normal = 0,
    EnPassant = 1,
    Castle = 2,
    Put = 3,
}

const MOVE_TYPE_OFFSET: u8 = ROLE_OFFSET + 3;
const ROLE_OFFSET: u8 = FROM_OFFSET + 6;
const FROM_OFFSET: u8 = CAPTURE_OFFSET + 3;
const CAPTURE_OFFSET: u8 = TO_OFFSET + 6;
const TO_OFFSET: u8 = PROMOTION_OFFSET + 3;
const PROMOTION_OFFSET: u8 = 0;

#[inline(always)]
fn set_move(entry: &mut PackedRep, mv: Move) {
    match mv {
        Move::Normal {
            role,
            from,
            capture,
            to,
            promotion,
        } => {
            let tp = MoveType::Normal as u128;
            let capture = match capture {
                Some(piece) => piece as u128,
                None => 0,
            };
            let promotion = match promotion {
                Some(piece) => piece as u128,
                None => 0,
            };
            *entry |= tp << MOVE_TYPE_OFFSET;
            *entry |= (role as u128) << ROLE_OFFSET;
            *entry |= (from as u128) << FROM_OFFSET;
            *entry |= (capture as u128) << CAPTURE_OFFSET;
            *entry |= (to as u128) << TO_OFFSET;
            *entry |= (promotion as u128) << PROMOTION_OFFSET;
        }
        Move::EnPassant { from, to } => {
            let tp = MoveType::EnPassant as u128;
            *entry |= tp << MOVE_TYPE_OFFSET;
            *entry |= (from as u128) << FROM_OFFSET;
            *entry |= (to as u128) << TO_OFFSET;
        }
        Move::Castle { king, rook } => {
            let tp = MoveType::Castle as u128;
            *entry |= tp << MOVE_TYPE_OFFSET;
            *entry |= (king as u128) << FROM_OFFSET;
            *entry |= (rook as u128) << TO_OFFSET;
        }
        Move::Put { role, to } => {
            let tp = MoveType::Put as u128;
            *entry |= tp << MOVE_TYPE_OFFSET;
            *entry |= (role as u128) << ROLE_OFFSET;
            *entry |= (to as u128) << TO_OFFSET;
        }
    }
}

#[inline]
pub fn get_move(entry: PackedRep) -> Move {
    let tp = (entry >> MOVE_TYPE_OFFSET) & 0b11;
    let role = (entry >> ROLE_OFFSET) & 0b111;
    let from = (entry >> FROM_OFFSET) & 0b111111;
    let capture = (entry >> CAPTURE_OFFSET) & 0b111;
    let to = (entry >> TO_OFFSET) & 0b111111;
    let promotion = (entry >> PROMOTION_OFFSET) & 0b111;

    match tp {
        0 => Move::Normal {
            role: get_role(role),
            from: get_square(from),
            to: get_square(to),
            capture: get_option_role(capture),
            promotion: get_option_role(promotion),
        },
        1 => Move::EnPassant {
            from: get_square(from),
            to: get_square(to),
        },
        2 => Move::Castle {
            king: get_square(from),
            rook: get_square(to),
        },
        3 => Move::Put {
            role: get_role(role),
            to: get_square(to),
        },
        _ => unreachable!(),
    }
}

fn get_role(input: u128) -> Role {
    match input {
        1 => Role::Pawn,
        2 => Role::Knight,
        3 => Role::Bishop,
        4 => Role::Rook,
        5 => Role::Queen,
        6 => Role::King,
        _ => unreachable!(),
    }
}

fn get_square(input: u128) -> Square {
    match input {
        i @ 0..64 => unsafe { std::mem::transmute(i as u8) },
        _ => unreachable!(),
    }
}

fn get_option_role(input: u128) -> Option<Role> {
    match input {
        0 => None,
        1 => Some(Role::Pawn),
        2 => Some(Role::Knight),
        3 => Some(Role::Bishop),
        4 => Some(Role::Rook),
        5 => Some(Role::Queen),
        6 => Some(Role::King),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use shakmaty::{fen::Fen, CastlingMode, Chess, EnPassantMode, Position};
    use std::str::FromStr;

    use super::*;

    fn parse_fen(fen_str: &str) -> Chess {
        Fen::from_str(fen_str)
            .unwrap()
            .into_position(CastlingMode::Standard)
            .unwrap()
    }

    #[test]
    fn test_simple() {
        let fens = include_str!("../../assets/fens.txt");

        for line in fens.lines() {
            let pos = parse_fen(line);
            let zobrist_hash = pos.zobrist_hash::<Zobrist64>(EnPassantMode::Legal);
            println!("zobrist_hash: {:016x}", zobrist_hash.0);
            let depth = rand::random::<u8>();
            let score = rand::random::<i32>();
            let bound = match rand::random::<u8>() % 3 {
                0 => Bound::Exact,
                1 => Bound::Upper,
                2 => Bound::Lower,
                _ => unreachable!(),
            };
            let mv = pos.legal_moves().first().unwrap().clone();

            let packed_rep = pack(zobrist_hash, mv, score, depth, bound);

            assert!(matches_zobrist(packed_rep, zobrist_hash));
            assert_eq!(get_depth(packed_rep), depth);
            assert_eq!(get_score(packed_rep), score);
            assert_eq!(get_bound(packed_rep), bound);
            assert_eq!(get_move(packed_rep), mv);
        }
    }
}
