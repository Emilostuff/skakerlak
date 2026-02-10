use crate::eval::phase::Phase;
use shakmaty::{Color, Piece, Role, Square};

/// Calculates the positional score of a piece on a given square for a given phase.
pub fn position_score(piece: Piece, square: Square, phase: &Phase) -> i32 {
    let mut sq = square as usize;
    if piece.color == Color::Black {
        sq = 63 - sq;
    }
    let (mg, eg) = match piece.role {
        Role::Pawn => (PAWN_TABLE[sq], PAWN_ENDGAME_TABLE[sq]),
        Role::Knight => (KNIGHT_TABLE[sq], KNIGHT_ENDGAME_TABLE[sq]),
        Role::Bishop => (BISHOP_TABLE[sq], BISHOP_ENDGAME_TABLE[sq]),
        Role::Rook => (ROOK_TABLE[sq], ROOK_ENDGAME_TABLE[sq]),
        Role::Queen => (QUEEN_TABLE[sq], QUEEN_ENDGAME_TABLE[sq]),
        Role::King => (KING_TABLE[sq], KING_ENDGAME_TABLE[sq]),
    };

    ((mg as f32 * phase.opening()) + (eg as f32 * phase.endgame())).round() as i32
}

// Pawns
#[rustfmt::skip]
const PAWN_TABLE: [i32; 64] = [
    0,    0,    0,    0,    0,    0,    0,    0,
    40,   40,   40,   40,   40,   40,   40,   40,
    15,   15,   20,   25,   25,   20,   15,   15,
    5,    5,   10,   20,   20,   10,    5,    5,
    0,    0,    5,   15,   15,    5,    0,    0,
    5,   -5,  -10,    0,    0,  -10,   -5,    5,
    5,   10,   10,  -15,  -15,   10,   10,    5,
    0,    0,    0,    0,    0,    0,    0,    0,
];

#[rustfmt::skip]
const PAWN_ENDGAME_TABLE: [i32; 64] = [
    0,    0,    0,    0,    0,    0,    0,    0,
    15,   15,   15,   15,   15,   15,   15,   15,
    30,   30,   30,   30,   30,   30,   30,   30,
    45,   45,   45,   45,   45,   45,   45,   45,
    60,   60,   60,   60,   60,   60,   60,   60,
    75,   75,   75,   75,   75,   75,   75,   75,
    90,   90,   90,   90,   90,   90,   90,   90,
    0,    0,    0,    0,    0,    0,    0,    0,
];

// Knights
#[rustfmt::skip]
const KNIGHT_TABLE: [i32; 64] = [
   -60,  -40,  -30,  -30,  -30,  -30,  -40,  -60,
   -40,  -20,    0,    5,    5,    0,  -20,  -40,
   -30,    5,   15,   20,   20,   15,    5,  -30,
   -30,   10,   20,   30,   30,   20,   10,  -30,
   -30,   10,   20,   30,   30,   20,   10,  -30,
   -30,    5,   15,   20,   20,   15,    5,  -30,
   -40,  -20,    0,    5,    5,    0,  -20,  -40,
   -60,  -40,  -30,  -30,  -30,  -30,  -40,  -60,
];

#[rustfmt::skip]
const KNIGHT_ENDGAME_TABLE: [i32; 64] = [
   -50,  -40,  -30,  -30,  -30,  -30,  -40,  -50,
   -40,  -25,  -10,   -5,   -5,  -10,  -25,  -40,
   -30,  -10,    0,    5,    5,    0,  -10,  -30,
   -30,   -5,    5,   10,   10,    5,   -5,  -30,
   -30,   -5,    5,   10,   10,    5,   -5,  -30,
   -30,  -10,    0,    5,    5,    0,  -10,  -30,
   -40,  -25,  -10,   -5,   -5,  -10,  -25,  -40,
   -50,  -40,  -30,  -30,  -30,  -30,  -40,  -50,
];

// Bishops
#[rustfmt::skip]
const BISHOP_TABLE: [i32; 64] = [
   -20,  -10,  -10,  -10,  -10,  -10,  -10,  -20,
   -10,    5,    0,    0,    0,    0,    5,  -10,
   -10,    0,   10,   15,   15,   10,    0,  -10,
   -10,    5,   15,   20,   20,   15,    5,  -10,
   -10,    5,   15,   20,   20,   15,    5,  -10,
   -10,    0,   10,   15,   15,   10,    0,  -10,
   -10,    5,    0,    0,    0,    0,    5,  -10,
   -20,  -10,  -10,  -10,  -10,  -10,  -10,  -20,
];

#[rustfmt::skip]
const BISHOP_ENDGAME_TABLE: [i32; 64] = [
   -10,    0,    0,    0,    0,    0,    0,  -10,
     0,    5,    5,    5,    5,    5,    5,    0,
     0,    5,   10,   10,   10,   10,    5,    0,
     0,    5,   10,   20,   20,   10,    5,    0,
     0,    5,   10,   20,   20,   10,    5,    0,
     0,    5,   10,   10,   10,   10,    5,    0,
     0,    5,    5,    5,    5,    5,    5,    0,
   -10,    0,    0,    0,    0,    0,    0,  -10,
];

// Rooks
#[rustfmt::skip]
const ROOK_TABLE: [i32; 64] = [
    0,    0,    5,   10,   10,    5,    0,    0,
    5,   10,   15,   20,   20,   15,   10,    5,
   -5,    0,    0,    5,    5,    0,    0,   -5,
   -5,    0,    0,    5,    5,    0,    0,   -5,
   -5,    0,    0,    5,    5,    0,    0,   -5,
   -5,    0,    0,    5,    5,    0,    0,   -5,
    5,   10,   10,   15,   15,   10,   10,    5,
    0,    0,    5,   10,   10,    5,    0,    0,
];

#[rustfmt::skip]
const ROOK_ENDGAME_TABLE: [i32; 64] = [
    0,    5,   10,   15,   15,   10,    5,    0,
    5,   10,   15,   20,   20,   15,   10,    5,
    0,    5,   10,   15,   15,   10,    5,    0,
    0,    5,   10,   15,   15,   10,    5,    0,
    0,    5,   10,   15,   15,   10,    5,    0,
    0,    5,   10,   15,   15,   10,    5,    0,
    5,   10,   15,   20,   20,   15,   10,    5,
    0,    5,   10,   15,   15,   10,    5,    0,
];

// Queens
#[rustfmt::skip]
const QUEEN_TABLE: [i32; 64] = [
   -30,  -20,  -20,  -10,  -10,  -20,  -20,  -30,
   -20,  -10,   -5,   -5,   -5,   -5,  -10,  -20,
   -20,   -5,    0,    0,    0,    0,   -5,  -20,
   -10,   -5,    0,    5,    5,    0,   -5,  -10,
   -10,   -5,    0,    5,    5,    0,   -5,  -10,
   -20,   -5,    0,    0,    0,    0,   -5,  -20,
   -20,  -10,   -5,   -5,   -5,   -5,  -10,  -20,
   -30,  -20,  -20,  -10,  -10,  -20,  -20,  -30,
];

#[rustfmt::skip]
const QUEEN_ENDGAME_TABLE: [i32; 64] = [
   -10,   -5,   -5,    0,    0,   -5,   -5,  -10,
    -5,    0,    5,    5,    5,    5,    0,   -5,
    -5,    5,   10,   10,   10,   10,    5,   -5,
     0,    5,   10,   15,   15,   10,    5,    0,
     0,    5,   10,   15,   15,   10,    5,    0,
    -5,    5,   10,   10,   10,   10,    5,   -5,
    -5,    0,    5,    5,    5,    5,    0,   -5,
   -10,   -5,   -5,    0,    0,   -5,   -5,  -10,
];

// Kings
#[rustfmt::skip]
const KING_TABLE: [i32; 64] = [
    20,   30,   10,    0,    0,   10,   30,   20,
    20,   20,    0,    0,    0,    0,   20,   20,
   -10,  -20,  -20,  -20,  -20,  -20,  -20,  -10,
   -20,  -30,  -30,  -40,  -40,  -30,  -30,  -20,
   -30,  -40,  -40,  -50,  -50,  -40,  -40,  -30,
   -30,  -40,  -40,  -50,  -50,  -40,  -40,  -30,
   -30,  -40,  -40,  -50,  -50,  -40,  -40,  -30,
   -30,  -40,  -40,  -50,  -50,  -40,  -40,  -30,
];

#[rustfmt::skip]
const KING_ENDGAME_TABLE: [i32; 64] = [
   -50,  -40,  -30,  -20,  -20,  -30,  -40,  -50,
   -30,  -20,  -10,    0,    0,  -10,  -20,  -30,
   -30,  -10,   20,   30,   30,   20,  -10,  -30,
   -30,  -10,   30,   40,   40,   30,  -10,  -30,
   -30,  -10,   30,   40,   40,   30,  -10,  -30,
   -30,  -10,   20,   30,   30,   20,  -10,  -30,
   -30,  -30,    0,    0,    0,    0,  -30,  -30,
   -50,  -30,  -30,  -30,  -30,  -30,  -30,  -50,
];

#[cfg(test)]
mod tests {
    use shakmaty::{fen::Fen, Board, CastlingMode, Chess, Position};
    use std::str::FromStr;

    use super::*;

    fn parse_fen(fen_str: &str) -> Chess {
        Fen::from_str(fen_str)
            .unwrap()
            .into_position(CastlingMode::Standard)
            .unwrap()
    }

    fn total_score_for_color(board: Board, phase: &Phase, color: Color) -> i32 {
        board
            .iter()
            .map(|(square, piece)| {
                if piece.color == color {
                    position_score(piece, square, phase)
                } else {
                    0
                }
            })
            .sum()
    }

    #[test]
    fn test_same_score_for_black_and_white() {
        let fens = include_str!("../../assets/fens.txt");

        for line in fens.lines() {
            let pos = parse_fen(line);
            let phase = Phase::new(&pos);

            let board = pos.board().clone();
            let board_rotated = {
                let mut new = board.clone();
                new.rotate_180();
                new.swap_colors();
                new
            };

            assert_eq!(
                total_score_for_color(board.clone(), &phase, Color::White),
                total_score_for_color(board_rotated, &phase, Color::Black),
                "Failed on {}",
                line
            );
        }
    }
}
