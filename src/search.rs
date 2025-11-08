use shakmaty::{Chess, Color, Move, Position, Role};

pub fn find_best_move(pos: &Chess) -> (i32, Option<Move>) {
    minimax(
        pos,
        6, // depth
        i32::MIN,
        i32::MAX,
    )
}

fn eval(pos: &Chess) -> i32 {
    if pos.is_checkmate() {
        return if pos.turn() == Color::White {
            -1000
        } else {
            1000
        };
    }

    // otherwise just count material difference
    let mut material_diff = 0;

    for (_, piece) in pos.board().iter() {
        let value = match piece.role {
            Role::Pawn => 1,
            Role::Knight => 3,
            Role::Bishop => 3,
            Role::Rook => 5,
            Role::Queen => 9,
            Role::King => 0,
        };
        material_diff += value * if piece.color == Color::White { 1 } else { -1 };
    }

    material_diff
}

pub fn minimax(pos: &Chess, depth: u8, mut alpha: i32, mut beta: i32) -> (i32, Option<Move>) {
    if depth == 0 || pos.is_checkmate() || pos.is_stalemate() {
        return (eval(pos), None);
    }

    let maximizing_player = pos.turn() == Color::White;
    let mut best_move = None;

    if maximizing_player {
        let mut max_eval = i32::MIN;
        for mv in pos.legal_moves() {
            let mut new_pos = pos.clone();
            new_pos.play_unchecked(&mv);
            let (eval, _) = minimax(&new_pos, depth - 1, alpha, beta);
            if eval > max_eval {
                max_eval = eval;
                best_move = Some(mv);
            }
            alpha = alpha.max(eval);
            if beta <= alpha {
                break;
            }
        }
        (max_eval, best_move)
    } else {
        let mut min_eval = i32::MAX;
        for mv in pos.legal_moves() {
            let mut new_pos = pos.clone();
            new_pos.play_unchecked(&mv);
            let (eval, _) = minimax(&new_pos, depth - 1, alpha, beta);
            if eval < min_eval {
                min_eval = eval;
                best_move = Some(mv);
            }
            beta = beta.min(eval);
            if beta <= alpha {
                break;
            }
        }
        (min_eval, best_move)
    }
}
