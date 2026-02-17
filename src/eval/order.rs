use shakmaty::{Move, MoveList};

use crate::eval::material::material_score;

/// Sorts a list of moves in order of how promising they are.
/// Start index allows for exluding a prefix of the list.
pub fn order(mut moves: MoveList, start_index: usize) -> MoveList {
    moves[start_index..].sort_by_key(|mv| -match mv {
        // Capture AND promote
        Move::Normal {
            capture: Some(_),
            promotion: Some(_),
            ..
        } => 10000,

        // Promote
        Move::Normal {
            promotion: Some(_), ..
        } => 9000,

        // En passant
        Move::EnPassant { .. } => 8000,

        // Capture
        Move::Normal {
            role,
            capture: Some(captured_role),
            ..
        } => 7000 + material_score(*captured_role) - material_score(*role),

        // Castling
        Move::Castle { .. } => 6000,

        // Regular move
        Move::Normal { .. } => 5000,

        // Not applicable in regular chess
        Move::Put { .. } => 0,
    });
    moves
}

#[cfg(test)]
mod tests {
    use shakmaty::{Move, Role, Square};

    use super::*;

    #[test]
    fn test_captures_first() {
        let mut moves = MoveList::new();

        moves.push(Move::Normal {
            role: Role::Bishop,
            from: Square::A5,
            capture: None,
            to: Square::B6,
            promotion: None,
        });

        moves.push(Move::Normal {
            role: Role::Bishop,
            from: Square::A5,
            capture: Some(Role::Knight),
            to: Square::B6,
            promotion: None,
        });

        moves = order(moves, 0);

        assert_eq!(moves[0].capture(), Some(Role::Knight));
        assert_eq!(moves[1].capture(), None);
    }

    #[test]
    fn preserve_first_move() {
        let mut moves = MoveList::new();

        moves.push(Move::Normal {
            role: Role::Bishop,
            from: Square::A5,
            capture: None,
            to: Square::B6,
            promotion: None,
        });

        moves.push(Move::Normal {
            role: Role::Bishop,
            from: Square::A5,
            capture: None,
            to: Square::B6,
            promotion: None,
        });

        moves.push(Move::Normal {
            role: Role::Bishop,
            from: Square::A5,
            capture: Some(Role::Knight),
            to: Square::B6,
            promotion: None,
        });

        moves = order(moves, 1);

        assert_eq!(moves[0].capture(), None);
        assert_eq!(moves[1].capture(), Some(Role::Knight));
        assert_eq!(moves[2].capture(), None);
    }
}
