use crate::{
    eval::order,
    search::{
        quiescence::quiescence,
        transposition::{Bound, TTEntry, TranspositionTable},
    },
};
use shakmaty::{
    zobrist::{Zobrist64, ZobristHash},
    Chess, EnPassantMode, Position,
};

pub fn negamax(
    board: &Chess,
    depth: u8,
    alpha: i32,
    beta: i32,
    ply: u8,
    tt: &mut TranspositionTable,
) -> i32 {
    // hash board state
    let hash = board.zobrist_hash::<Zobrist64>(EnPassantMode::Legal);

    // 1️⃣ Check for TT hit
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

    // 2️⃣ Terminal node
    if depth == 0 || board.is_game_over() {
        return quiescence(board, alpha, beta, ply);
    }

    let mut best_score = i32::MIN + 1;
    let mut best_move = None;
    let mut alpha = alpha;
    let alpha_orig = alpha;

    // 3️⃣ Generate legal moves
    let mut moves = board.legal_moves();

    // 4️⃣ Move ordering: TT best move first
    if let Some(tt_entry) = tt.lookup(hash) {
        if let Some(tt_best_move) = &tt_entry.best_move {
            if let Some(i) = moves.iter().position(|m| m == tt_best_move) {
                moves.swap(0, i);
            }
        }
    }

    moves = order::order(moves);

    for mv in moves {
        let mut new_pos = board.clone();
        new_pos.play_unchecked(&mv);
        let score = -negamax(&mut new_pos, depth - 1, -beta, -alpha, ply + 1, tt);

        if score > best_score {
            best_score = score;
            best_move = Some(mv); // <- track the move that actually gave best_score
        }

        alpha = alpha.max(score);
        if alpha >= beta {
            break; // beta cutoff
        }
    }

    // 8️⃣ Store TT entry
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
