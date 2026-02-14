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
