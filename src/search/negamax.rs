use crate::{
    eval::order,
    search::{
        quiescence::quiescence,
        transposition::{Bound, TTEntry, TranspositionTable},
    },
};
use shakmaty::{zobrist::Zobrist64, Chess, EnPassantMode, Position};

pub fn negamax(
    board: &Chess,
    depth: u8,
    alpha: i32,
    beta: i32,
    ply: u8,
    tt: &mut TranspositionTable,
    nodes: &mut u64,
    hash: Zobrist64,
) -> i32 {
    // Check for TT hit
    if let Some(entry) = tt.lookup(hash) {
        if entry.depth >= depth {
            match entry.bound {
                Bound::Exact => return entry.score,
                Bound::Lower if entry.score >= beta => return entry.score,
                Bound::Upper if entry.score <= alpha => return entry.score,
                _ => {} // otherwise fall through
            }
        }
    }

    // Increment nodes count
    *nodes += 1;

    // Terminal node
    if depth == 0 || board.is_game_over() {
        return quiescence(board, alpha, beta, ply);
    }

    let mut best_score = i32::MIN + 1;
    let mut best_move = None;
    let mut alpha = alpha;
    let alpha_orig = alpha;

    let mut moves = board.legal_moves();

    // Fetch best move from TT if present
    let mut order_start_index = 0;
    if let Some(tt_best_move) = tt.best_move(hash) {
        if let Some(i) = moves.iter().position(|m| m == &tt_best_move) {
            moves.swap(0, i);
            order_start_index = 1;
        }
    }

    // Sort moves
    moves = order::order(moves, order_start_index);

    for mv in moves {
        let mut new_pos = board.clone();
        new_pos.play_unchecked(mv.clone());
        let new_hash =
            match board.update_zobrist_hash::<Zobrist64>(hash, mv.clone(), EnPassantMode::Legal) {
                Some(h) => h,
                None => new_pos.zobrist_hash::<Zobrist64>(EnPassantMode::Legal),
            };
        let score = -negamax(
            &mut new_pos,
            depth - 1,
            -beta,
            -alpha,
            ply + 1,
            tt,
            nodes,
            new_hash,
        );

        if score > best_score {
            best_score = score;
            best_move = Some(mv); // <- track the move that actually gave best_score
        }

        alpha = alpha.max(score);
        if alpha >= beta {
            break; // beta cutoff
        }
    }

    // Store TT entry
    let bound = if best_score <= alpha_orig {
        Bound::Upper
    } else if best_score >= beta {
        Bound::Lower
    } else {
        Bound::Exact
    };

    tt.store(
        hash,
        TTEntry {
            score: best_score,
            depth,
            bound,
            best_move, // best move found at this node
        },
    );

    best_score
}
