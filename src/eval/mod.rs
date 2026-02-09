pub mod material;
pub mod order;
pub mod phase;
pub mod pst;

use crate::eval::{material::material_score, phase::Phase, pst::position_score};
use shakmaty::{Chess, Position};

/// Evaluates the current position.
pub fn evaluate(pos: &Chess, ply: u8) -> i32 {
    // First check for termination
    if pos.is_checkmate() {
        // Add one per ply to express mate in x moves (offset +1 due to i32 range assymetry).
        return i32::MIN + 1 + ply as i32;
    } else if pos.is_game_over() {
        // Draw
        return 0;
    }

    // Calculate phase
    let phase = Phase::new(pos);

    // Accumulator
    let mut diff = 0;

    for (square, piece) in pos.board().iter() {
        let score = material_score(piece.role) + position_score(piece, square, &phase);

        if piece.color == pos.turn() {
            diff += score;
        } else {
            diff -= score;
        }
    }

    diff
}
