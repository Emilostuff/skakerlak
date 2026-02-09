pub mod order;
pub mod pst;

use shakmaty::{Chess, Position, Role};

pub fn evaluate(pos: &Chess, ply: u8) -> i32 {
    // check for end of game
    if pos.is_checkmate() {
        return i32::MIN + 1 + ply as i32;
    } else if pos.is_game_over() {
        return 0;
    }

    let phase = pst::calculate_phase(pos);

    let mut material_diff = 0;

    for (square, piece) in pos.board().iter() {
        let material = match piece.role {
            Role::Pawn => 100,
            Role::Knight => 320,
            Role::Bishop => 330,
            Role::Rook => 500,
            Role::Queen => 900,
            Role::King => 0,
        };

        let position = pst::position_score(piece, square, phase);

        let total = material + position;

        if piece.color == pos.turn() {
            material_diff += total;
        } else {
            material_diff -= total;
        }
    }

    material_diff
}
