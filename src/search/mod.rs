pub mod negamax;
pub mod quiescence;
pub mod transposition;

use crate::{
    eval::order,
    search::transposition::{Bound, TTEntry, TranspositionTable},
    SearchCommand, SearchControl, SearchInfo,
};
use crossbeam_channel::{Receiver, Sender};
use negamax::negamax;
use shakmaty::{zobrist::Zobrist64, Chess, EnPassantMode, Move, Position};

/// Executes search tasks.
pub struct Searcher {
    cmd_rx: Receiver<SearchCommand>,
    info_tx: Sender<SearchInfo>,
    tt: TranspositionTable,
}

impl Searcher {
    pub fn new(cmd_rx: Receiver<SearchCommand>, info_tx: Sender<SearchInfo>) -> Self {
        Searcher {
            cmd_rx,
            info_tx,
            tt: TranspositionTable::new(100_000_000),
        }
    }

    /// Run the searcher
    pub fn run(mut self) {
        loop {
            match self.cmd_rx.recv() {
                Ok(SearchCommand::Start { position, control }) => self.search(position, control),
                Ok(SearchCommand::Stop) => (),
                Ok(SearchCommand::Quit) | Err(_) => break,
            }
        }
    }

    fn search(&mut self, position: Chess, control: SearchControl) {
        let start_time = std::time::Instant::now();
        let mut best_move = position
            .legal_moves()
            .first()
            .expect("No legal moves found")
            .clone();
        let mut best_score = i32::MIN + 1;

        // Determine search constraints
        let (max_depth, time_limit) = match control {
            SearchControl::ToDepth(depth) => (depth, u64::MAX),
            SearchControl::TimeLimit(time_limit) => (u8::MAX, time_limit),
        };

        // Hash start position
        let hash = position.zobrist_hash::<Zobrist64>(EnPassantMode::Legal);

        // Iterative deepening loop
        'outer: for depth in 1..=max_depth {
            let mut iteration_best_move = None;
            let mut iteration_best_score = i32::MIN + 1;
            let mut nodes = 0;

            // Generate moves from position
            let mut moves = position.legal_moves();

            // Fetch best move from TT if present
            let mut order_start_index = 0;
            if let Some(tt_best_move) = self.tt.best_move(hash) {
                if let Some(i) = moves.iter().position(|m| m == &tt_best_move) {
                    moves.swap(0, i);
                    order_start_index = 1;
                }
            }

            // Sort moves
            moves = order::order(moves, order_start_index);

            for mv in moves {
                // Get resulting position after move
                let mut new_pos = position.clone();
                new_pos.play_unchecked(mv.clone());
                let hash = new_pos.zobrist_hash(EnPassantMode::Legal);

                // Search from here
                let score = -negamax(
                    &new_pos,
                    depth - 1,
                    i32::MIN + 1,
                    i32::MAX,
                    1,
                    &mut self.tt,
                    &mut nodes,
                    hash,
                );

                // Update results if score has improved
                if score > iteration_best_score {
                    iteration_best_score = score;
                    iteration_best_move = Some(mv);
                }

                // Check if allowed time has run out
                let elapsed = start_time.elapsed();
                if elapsed > std::time::Duration::from_millis(time_limit) {
                    break 'outer;
                }

                // Check for external interrupts
                match self.cmd_rx.try_recv() {
                    Ok(SearchCommand::Start { .. }) | Ok(SearchCommand::Stop) => break 'outer,
                    Ok(SearchCommand::Quit) => return,
                    _ => (),
                };
            }

            // Update global best from iteration
            if let Some(iter_move) = iteration_best_move {
                best_move = iter_move;
                best_score = iteration_best_score;
            }

            // Store result in tt
            self.tt.store(
                hash,
                TTEntry {
                    score: best_score,
                    depth,
                    bound: Bound::Exact,
                    best_move: Some(best_move.clone()), // best move found at this node
                },
            );

            // Construct pv
            let pv = self.tt.pv(position.clone(), hash, depth);

            // Send info from iteration
            self.send_info(depth, pv, best_score, nodes);
        }

        // Output best move
        self.info_tx.send(SearchInfo::BestMove(best_move)).unwrap();
    }

    fn send_info(&self, depth: u8, pv: Vec<Move>, score: i32, nodes: u64) {
        self.info_tx
            .send(SearchInfo::Info {
                depth,
                pv,
                score,
                nodes,
            })
            .unwrap();
    }

    pub fn reset(&mut self) {
        self.tt = TranspositionTable::new(100_000_000);
    }
}
