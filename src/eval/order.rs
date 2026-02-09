use shakmaty::Move;

pub fn order(mut moves: Vec<Move>) -> Vec<Move> {
    moves.sort_by_key(|mv| -match mv {
        Move::Normal {
            capture: Some(_),
            promotion: Some(_),
            ..
        } => 10,
        Move::Normal {
            capture: None,
            promotion: Some(_),
            ..
        } => 7,
        Move::EnPassant { .. } => 6,
        Move::Normal {
            capture: Some(_),
            promotion: None,
            ..
        } => 5,
        Move::Normal {
            capture: None,
            promotion: None,
            ..
        } => 2,
        Move::Castle { .. } => 2,
        _ => 0,
    });
    moves
}
