use shakmaty::{
    zobrist::{Zobrist64, ZobristHash},
    Chess, EnPassantMode, Move, Position,
};
use std::collections::HashMap;

#[derive(Copy, Clone, Debug)]
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

    pub fn store(&mut self, key: Zobrist64, entry: TTEntry) {
        if self.table.len() >= self.max_size {
            // simple replacement: remove random or first inserted
            // advanced: use depth-prefer replacement
            let first_key = *self.table.keys().next().unwrap();
            self.table.remove(&first_key);
        }
        self.table.insert(key, entry);
    }

    pub fn best_move(&self, hash: Zobrist64) -> Option<Move> {
        self.table
            .get(&hash)
            .and_then(|entry| entry.best_move.clone())
    }

    pub fn pv(&self, mut position: Chess, hash: Zobrist64, max_depth: u8) -> Vec<Move> {
        let mut pv = Vec::new();
        let mut depth = max_depth;
        let mut current_hash = hash;

        while let Some(mv) = self.best_move(current_hash) {
            position.play_unchecked(&mv);
            pv.push(mv);
            current_hash = position.zobrist_hash::<Zobrist64>(EnPassantMode::Legal);
            depth -= 1;
            if depth == 0 {
                break;
            }
        }

        pv
    }
}
