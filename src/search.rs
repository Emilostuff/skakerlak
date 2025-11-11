use once_cell::sync::Lazy;
use rand::seq::IndexedRandom;
use shakmaty::{Chess, Color, Move, Position, Role};

use crate::book::Book;

pub fn find_best_move(pos: &Chess) -> (i32, Option<Move>) {
    static BOOK: Lazy<Book> = Lazy::new(|| {
        let book_data: &[u8] = include_bytes!("../human.bin");
        Book::from_bytes(book_data)
    });

    let moves = BOOK.moves(pos);

    if !moves.is_empty() {
        // return random move from book
        let random_move = moves.choose(&mut rand::rng()).unwrap();
        dbg!(&random_move);
        return (8800, Some(random_move.to_move(pos).unwrap()));
    }

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
            -42000
        } else {
            42000
        };
    } else if pos.has_insufficient_material(pos.turn()) || pos.is_stalemate() {
        return 0;
    }

    // otherwise just count material difference
    let mut material_diff = 0;

    for (_, piece) in pos.board().iter() {
        let value = match piece.role {
            Role::Pawn => 100,
            Role::Knight => 320,
            Role::Bishop => 330,
            Role::Rook => 500,
            Role::Queen => 900,
            Role::King => 0,
        };
        material_diff += value * if piece.color == Color::White { 1 } else { -1 };
    }

    material_diff
}

pub fn minimax(pos: &Chess, depth: u8, mut alpha: i32, mut beta: i32) -> (i32, Option<Move>) {
    if depth == 0 || pos.is_game_over() {
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
