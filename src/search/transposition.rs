use shakmaty::{zobrist::Zobrist64, Chess, EnPassantMode, Move, Position};
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Bound {
    Exact,
    Lower,
    Upper,
}

#[derive(Clone, Debug)]
pub struct TTEntry {
    pub score: i32,
    pub depth: u8,
    pub bound: Bound,
    pub best_move: Option<Move>,
}

pub struct TranspositionTable {
    table: HashMap<Zobrist64, TTEntry>,
    max_size: usize,
}

impl TranspositionTable {
    pub fn new(max_size: usize) -> Self {
        Self {
            table: HashMap::with_capacity(max_size),
            max_size,
        }
    }

    pub fn lookup(&self, key: Zobrist64) -> Option<&TTEntry> {
        self.table.get(&key)
    }

    pub fn store(
        &mut self,
        key: Zobrist64,
        score: i32,
        depth: u8,
        bound: Bound,
        best_move: Option<Move>,
    ) {
        if self.table.len() >= self.max_size {
            // simple replacement: remove random or first inserted
            // advanced: use depth-prefer replacement
            let first_key = *self.table.keys().next().unwrap();
            self.table.remove(&first_key);
        }
        self.table.insert(
            key,
            TTEntry {
                score,
                depth,
                bound,
                best_move,
            },
        );
    }

    pub fn best_move(&self, hash: Zobrist64) -> Option<Move> {
        self.table
            .get(&hash)
            .and_then(|entry| entry.best_move.clone())
    }

    pub fn pv(&self, mut pos: Chess, mut best_move: Option<Move>, mut depth: u8) -> Vec<Move> {
        let mut pv = Vec::new();

        while let Some(mv) = best_move {
            // Add move to pv
            pv.push(mv.clone());

            // Update position by playing move
            pos.play_unchecked(mv);
            depth -= 1;

            // Stop if depth has been reached
            if depth == 0 {
                break;
            }

            // Find best move at new position
            best_move = self.best_move(pos.zobrist_hash::<Zobrist64>(EnPassantMode::Legal));
        }

        pv
    }
}
