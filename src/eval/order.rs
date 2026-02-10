use shakmaty::{Move, MoveList};

/// Sorts a list of moves in order of how promising they are.
pub fn order(mut moves: MoveList) -> MoveList {
    moves.sort_by_key(|mv| -match mv {
        // Capture AND promote
        Move::Normal {
            capture: Some(_),
            promotion: Some(_),
            ..
        } => 10,

        // Promote
        Move::Normal {
            promotion: Some(_), ..
        } => 7,

        // En passant
        Move::EnPassant { .. } => 6,

        // Capture
        Move::Normal {
            capture: Some(_), ..
        } => 5,

        // Castling
        Move::Castle { .. } => 3,

        // Regular move
        Move::Normal { .. } => 2,

        // Not applicable in regular chess
        Move::Put { .. } => 0,
    });
    moves
}
