pub mod negamax;
pub mod quiescence;
pub mod transposition;

use crate::{
    eval::order,
    search::transposition::{Bound, FastTranspositionTable, TranspositionTable},
    SearchCommand, SearchControl, SearchInfo,
};
use crossbeam_channel::{Receiver, Sender};
use negamax::negamax;
use shakmaty::{zobrist::Zobrist64, Chess, EnPassantMode, Move, Position};

/// Executes search tasks.
pub struct Searcher {
    cmd_rx: Receiver<SearchCommand>,
    info_tx: Sender<SearchInfo>,
    tt: FastTranspositionTable,
}

#[derive(Clone)]
pub struct Best {
    pub move_: Move,
    pub score: i32,
}

impl Searcher {
    pub fn new(cmd_rx: Receiver<SearchCommand>, info_tx: Sender<SearchInfo>) -> Self {
        Searcher {
            cmd_rx,
            info_tx,
            tt: FastTranspositionTable::new(28),
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
        let mut best = Best {
            move_: position
                .legal_moves()
                .first()
                .expect("No legal moves found")
                .clone(),
            score: i32::MIN + 1,
        };

        // Determine search constraints
        let (max_depth, time_limit) = match control {
            SearchControl::ToDepth(depth) => (depth, u64::MAX),
            SearchControl::TimeLimit(time_limit) => (u8::MAX, time_limit),
        };

        // Hash start position
        let hash = position.zobrist_hash::<Zobrist64>(EnPassantMode::Legal);

        // Iterative deepening loop
        'outer: for depth in 1..=max_depth {
            let mut iteration_best = Best {
                move_: best.move_.clone(),
                score: i32::MIN + 1,
            };

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
                    -iteration_best.score,
                    1,
                    &mut self.tt,
                    &mut nodes,
                    hash,
                );

                // Update results if score has improved
                if score > iteration_best.score {
                    iteration_best = Best {
                        score,
                        move_: mv.clone(),
                    };
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
            best = iteration_best;

            // Store result in tt
            self.tt.store(
                hash,
                best.score,
                depth,
                Bound::Exact,
                best.move_.clone(), // best move found at this node
            );

            // Construct pv
            let pv = self
                .tt
                .pv(position.clone(), Some(best.move_.clone()), depth);

            // Send info from iteration
            self.send_info(depth, pv, best.score, nodes);
        }

        // Output best move
        self.info_tx.send(SearchInfo::BestMove(best.move_)).unwrap();
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
        self.tt = FastTranspositionTable::new(28);
    }
}
