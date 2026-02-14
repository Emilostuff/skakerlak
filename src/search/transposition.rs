use std::sync::atomic::Ordering;

use crate::search::pack::*;
use portable_atomic::AtomicU128;
use shakmaty::{zobrist::Zobrist64, Chess, EnPassantMode, Move, Position};

pub trait TranspositionTable {
    fn lookup(&self, key: Zobrist64) -> Option<TTEntry>;
    fn store(&self, key: Zobrist64, score: i32, depth: u8, bound: Bound, best_move: Move);
    fn best_move(&self, key: Zobrist64) -> Option<Move>;
    fn pv(&self, pos: Chess, best_move: Option<Move>, depth: u8) -> Vec<Move>;
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
    table: Vec<AtomicU128>,
    size_power: u8,
}

impl FastTranspositionTable {
    pub fn new(size_power: u8) -> Self {
        let table_len = 1 << size_power;
        let mut table = Vec::with_capacity(table_len);
        for _ in 0..table_len {
            table.push(AtomicU128::new(0));
        }
        Self { table, size_power }
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
        let entry = self.table[index].load(Ordering::Relaxed);
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
    fn store(&self, key: Zobrist64, score: i32, depth: u8, bound: Bound, best_move: Move) {
        let index = self.index(key);
        self.table[index].store(pack(key, best_move, score, depth, bound), Ordering::Relaxed);
    }

    #[inline(always)]
    fn best_move(&self, key: Zobrist64) -> Option<Move> {
        let index = self.index(key);
        let entry = self.table[index].load(Ordering::Relaxed);
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
}
