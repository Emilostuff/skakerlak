use shakmaty::{zobrist::Zobrist64, Move};
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
    table: HashMap<Zobrist64, TTEntry>, // simple first implementation
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
}
