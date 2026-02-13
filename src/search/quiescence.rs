use crate::eval::evaluate;
use shakmaty::{Chess, Position};

pub fn quiescence(board: &Chess, alpha: i32, beta: i32, ply: u8) -> i32 {
    let stand_pat = evaluate(board, ply);
    if stand_pat >= beta {
        return beta;
    }
    let mut alpha = alpha.max(stand_pat);

    for mv in board.capture_moves() {
        let mut new_board = board.clone();
        new_board.play_unchecked(mv.clone());
        let score = -quiescence(&new_board, -beta, -alpha, ply + 1);
        if score >= beta {
            return beta;
        }
        alpha = alpha.max(score);
    }

    alpha
}
