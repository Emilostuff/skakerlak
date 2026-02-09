use crate::eval::{evaluate, order::order};
use shakmaty::{Chess, Move, Position};

pub fn negamax(
    pos: &Chess,
    depth: u8,
    mut alpha: i32,
    beta: i32,
    ply: u8,
    nodes: &mut u64,
    mut former_pv: &[Move],
) -> (i32, Vec<Move>) {
    *nodes += 1;

    if depth == 0 || pos.is_game_over() {
        return (evaluate(pos, ply), vec![]);
    }

    // Generate moves and order them
    let mut moves = order(pos.legal_moves().into_iter().collect());

    // if pv move exists, move it to the beginning of the list
    if let Some(mv) = former_pv.first() {
        moves.retain(|m| m != mv);
        moves.insert(0, mv.clone());
    }

    let mut best_score = i32::MIN + 1;
    let mut best_line = vec![];

    for (i, mv) in moves.iter().enumerate() {
        // If pv move is played, pass on rest of the pv
        if i == 0 && former_pv.first().is_some() {
            former_pv = &former_pv[1..];
        } else {
            former_pv = &[];
        }

        let mut new_pos = pos.clone();
        new_pos.play_unchecked(&mv);

        let (score, child_pv) = negamax(
            &new_pos,
            depth - 1,
            -beta,
            -alpha,
            ply + 1,
            nodes,
            former_pv,
        );

        let score = -score;

        if score > best_score {
            best_score = score;
            best_line = vec![mv.clone()];
            best_line.extend(child_pv);
        }

        alpha = alpha.max(score);
        if alpha >= beta {
            break;
        }
    }

    (best_score, best_line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::{Chess, Move};

    // #[test]
    // fn test_negamax() {
    //     let pos = Chess::default();
    //     let depth = 3;
    //     let alpha = i32::MIN;
    //     let beta = i32::MAX;
    //     let ply = 0;
    //     let mut nodes = 0;
    //     let former_pv = &[];

    //     let (score, line) = negamax(&pos, depth, alpha, beta, ply, &mut nodes, former_pv);

    //     assert_eq!(score, 0);
    //     assert_eq!(line, vec![]);
    // }
}
