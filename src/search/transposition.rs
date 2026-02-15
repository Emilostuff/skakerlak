use crate::search::pack::*;
use shakmaty::{zobrist::Zobrist64, Chess, EnPassantMode, Move, Position};

use crate::search::pack::PackedRep;

pub trait TranspositionTable {
    fn lookup(&self, key: Zobrist64) -> Option<TTEntry>;
    fn store(&mut self, key: Zobrist64, score: i32, depth: u8, bound: Bound, best_move: Move);
    fn best_move(&self, key: Zobrist64) -> Option<Move>;
    fn pv(&self, pos: Chess, best_move: Option<Move>, depth: u8) -> Vec<Move>;
    fn clear(&mut self);
}

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
    pub best_move: Move,
}

pub struct FastTranspositionTable {
    table: Vec<PackedRep>,
    size_power: u8,
}

impl FastTranspositionTable {
    pub fn new(size_power: u8) -> Self {
        Self {
            table: vec![0; 1 << size_power],
            size_power,
        }
    }

    #[inline(always)]
    fn index(&self, key: Zobrist64) -> usize {
        (key.0 >> (64 - self.size_power)) as usize
    }
}

impl TranspositionTable for FastTranspositionTable {
    #[inline(always)]
    fn lookup(&self, key: Zobrist64) -> Option<TTEntry> {
        let index = self.index(key);
        let entry = self.table[index];
        if matches_zobrist(entry, key) {
            return Some(TTEntry {
                score: get_score(entry),
                depth: get_depth(entry),
                bound: get_bound(entry),
                best_move: get_move(entry),
            });
        }
        None
    }

    #[inline(always)]
    fn store(&mut self, key: Zobrist64, score: i32, depth: u8, bound: Bound, best_move: Move) {
        let index = self.index(key);
        self.table[index] = pack(key, best_move, score, depth, bound)
    }

    #[inline(always)]
    fn best_move(&self, key: Zobrist64) -> Option<Move> {
        let index = self.index(key);
        let entry = self.table[index];
        if matches_zobrist(entry, key) {
            return Some(get_move(entry));
        }
        None
    }

    #[inline(always)]
    fn pv(&self, mut pos: Chess, mut best_move: Option<Move>, mut depth: u8) -> Vec<Move> {
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

    fn clear(&mut self) {
        self.table = vec![0; 1 << self.size_power]
    }
}
