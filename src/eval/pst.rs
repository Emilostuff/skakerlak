use shakmaty::{ByRole, Chess, Color, Piece, Position, Role, Square};

pub fn calculate_phase(pos: &Chess) -> f32 {
    let phase: u8 = pos
        .board()
        .material()
        .iter()
        .map(
            |ByRole {
                 knight,
                 bishop,
                 rook,
                 queen,
                 ..
             }| { knight + bishop + rook * 2 + queen * 4 },
        )
        .sum();
    let max_phase = 24; // Starting phase value
    1.0 - (phase as f32 / max_phase as f32) // 1.0 = endgame, 0.0 = opening
}

pub fn position_score(piece: Piece, square: Square, phase: f32) -> i32 {
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

    // make phase more conservative
    let phase = phase.powf(1.5);

    ((mg as f32 * (1.0 - phase)) + (eg as f32 * phase)).round() as i32
}

#[rustfmt::skip]
const PAWN_TABLE: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
     50,  50,  50,  50,  50,  50,  50,  50,
     10,  10,  20,  30,  30,  20,  10,  10,
      5,   5,  10,  25,  25,  10,   5,   5,
      0,   0,   0,  20,  20,   0,   0,   0,
      5,  -5, -10,   0,   0, -10,  -5,   5,
      5,  10,  10, -20, -20,  10,  10,   5,
      0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
const BISHOP_TABLE: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,  10,  10,   5,   0, -10,
    -10,   5,   5,  10,  10,   5,   5, -10,
    -10,   0,  10,  10,  10,  10,   0, -10,
    -10,  10,  10,  10,  10,  10,  10, -10,
    -10,   5,   0,   0,   0,   0,   5, -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
];

#[rustfmt::skip]
const KNIGHT_TABLE: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,   0,   0,   0,   0, -20, -40,
    -30,   0,  10,  15,  15,  10,   0, -30,
    -30,   5,  15,  20,  20,  15,   5, -30,
    -30,   0,  15,  20,  20,  15,   0, -30,
    -30,   5,  10,  15,  15,  10,   5, -30,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];

#[rustfmt::skip]
const ROOK_TABLE: [i32; 64] = [
     0,   0,   0,   0,   0,   0,   0,   0,
     5,  10,  10,  10,  10,  10,  10,   5,
    -5,   0,   0,   0,   0,   0,   0,  -5,
    -5,   0,   0,   0,   0,   0,   0,  -5,
    -5,   0,   0,   0,   0,   0,   0,  -5,
    -5,   0,   0,   0,   0,   0,   0,  -5,
     5,   0,   0,   0,   0,   0,   0,   5,
     0,   0,   0,   5,   5,   0,   0,   0,
];

#[rustfmt::skip]
const QUEEN_TABLE: [i32; 64] = [
    -20, -10, -10,  -5,  -5, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,   5,   5,   5,   0, -10,
     -5,   0,   5,   5,   5,   5,   0,  -5,
      0,   0,   5,   5,   5,   5,   0,  -5,
    -10,   5,   5,   5,   5,   5,   0, -10,
    -10,   0,   5,   0,   0,   0,   0, -10,
    -20, -10, -10,  -5,  -5, -10, -10, -20,
];

#[rustfmt::skip]
const KING_TABLE: [i32; 64] = [
     20,  30,  10,   0,   0,  10,  30,  20,
     20,  20,   0,   0,   0,   0,  20,  20,
    -10, -20, -20, -20, -20, -20, -20, -10,
    -20, -30, -30, -40, -40, -30, -30, -20,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
];

#[rustfmt::skip]
const PAWN_ENDGAME_TABLE: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
     10,  10,  10,  10,  10,  10,  10,  10,
     20,  20,  20,  20,  20,  20,  20,  20,
     30,  30,  30,  30,  30,  30,  30,  30,
     40,  40,  40,  40,  40,  40,  40,  40,
     50,  50,  50,  50,  50,  50,  50,  50,
     60,  60,  60,  60,  60,  60,  60,  60,
      0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
const KNIGHT_ENDGAME_TABLE: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,   0,   0,   0,   0, -20, -40,
    -30,   0,  10,  15,  15,  10,   0, -30,
    -30,   5,  15,  20,  20,  15,   5, -30,
    -30,   0,  15,  20,  20,  15,   0, -30,
    -30,   5,  10,  15,  15,  10,   5, -30,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];

#[rustfmt::skip]
const BISHOP_ENDGAME_TABLE: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,  10,  10,   5,   0, -10,
    -10,   5,   5,  10,  10,   5,   5, -10,
    -10,   0,  10,  10,  10,  10,   0, -10,
    -10,  10,  10,  10,  10,  10,  10, -10,
    -10,   5,   0,   0,   0,   0,   5, -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
];

#[rustfmt::skip]
const ROOK_ENDGAME_TABLE: [i32; 64] = [
     0,   0,   0,   0,   0,   0,   0,   0,
     5,  10,  10,  10,  10,  10,  10,   5,
    -5,   0,   0,   0,   0,   0,   0,  -5,
    -5,   0,   0,   0,   0,   0,   0,  -5,
    -5,   0,   0,   0,   0,   0,   0,  -5,
    -5,   0,   0,   0,   0,   0,   0,  -5,
     5,   0,   0,   0,   0,   0,   0,   5,
     0,   0,   0,   5,   5,   0,   0,   0,
];

#[rustfmt::skip]
const QUEEN_ENDGAME_TABLE: [i32; 64] = [
    -20, -10, -10,  -5,  -5, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,   5,   5,   5,   0, -10,
     -5,   0,   5,   5,   5,   5,   0,  -5,
      0,   0,   5,   5,   5,   5,   0,  -5,
    -10,   5,   5,   5,   5,   5,   0, -10,
    -10,   0,   5,   0,   0,   0,   0, -10,
    -20, -10, -10,  -5,  -5, -10, -10, -20,
];

#[rustfmt::skip]
const KING_ENDGAME_TABLE: [i32; 64] = [
    -50, -40, -30, -20, -20, -30, -40, -50,
    -30, -20, -10,   0,   0, -10, -20, -30,
    -30, -10,  20,  30,  30,  20, -10, -30,
    -30, -10,  30,  40,  40,  30, -10, -30,
    -30, -10,  30,  40,  40,  30, -10, -30,
    -30, -10,  20,  30,  30,  20, -10, -30,
    -30, -30,   0,   0,   0,   0, -30, -30,
    -50, -30, -30, -30, -30, -30, -30, -50,
];

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::{fen::Fen, CastlingMode, Chess};
    use std::str::FromStr;

    fn parse_fen(fen_str: &str) -> Chess {
        Fen::from_str(fen_str)
            .unwrap()
            .into_position(CastlingMode::Standard)
            .unwrap()
    }

    #[test]
    fn test_phase_opening() {
        // Standard chess opening position
        let pos = Chess::default();
        let phase = calculate_phase(&pos);
        assert!(
            phase.abs() < 1e-6,
            "Opening phase should be 0.0, got {}",
            phase
        );
    }

    #[test]
    fn test_phase_endgame() {
        // Only kings and pawns
        let pos = parse_fen("4k3/8/8/3p4/3P4/8/8/4K3 w - - 0 1");
        let phase = calculate_phase(&pos);
        assert!(
            (phase - 1.0).abs() < 1e-6,
            "Endgame phase should be 1.0, got {}",
            phase
        );
    }

    #[test]
    fn test_phase_middlegame() {
        // Remove both queens from opening position
        let pos = parse_fen("rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNB1KBNR w KQkq - 0 1");
        let phase = calculate_phase(&pos);
        assert!(
            phase > 0.0 && phase < 1.0,
            "Middlegame phase should be between 0.0 and 1.0, got {}",
            phase
        );
    }
}
