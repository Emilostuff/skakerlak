use shakmaty::{Move, MoveList};

/// Sorts a list of moves in order of how promising they are.
/// Start index allows for exluding a prefix of the list.
pub fn order(mut moves: MoveList, start_index: usize) -> MoveList {
    moves[start_index..].sort_by_key(|mv| match mv {
        // Capture AND promote
        Move::Normal {
            capture: Some(_),
            promotion: Some(_),
            ..
        } => 0,

        // Promote
        Move::Normal {
            promotion: Some(_), ..
        } => 1,

        // En passant
        Move::EnPassant { .. } => 2,

        // Capture
        Move::Normal {
            capture: Some(_), ..
        } => 3,

        // Castling
        Move::Castle { .. } => 4,

        // Regular move
        Move::Normal { .. } => 5,

        // Not applicable in regular chess
        Move::Put { .. } => 6,
    });
    moves
}
