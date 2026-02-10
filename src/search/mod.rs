pub mod negamax;
pub mod quiescence;
pub mod transposition;

use negamax::negamax;

use crate::{
    eval::order, search::transposition::TranspositionTable, SearchCommand, SearchControl,
    SearchInfo,
};
use crossbeam_channel::{Receiver, Sender};
use shakmaty::{
    zobrist::{Zobrist64, ZobristHash},
    Chess, EnPassantMode, Position,
};

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
            tt: TranspositionTable::new(100000),
        }
    }

    pub fn run(mut self) {
        loop {
            match self.cmd_rx.recv().unwrap() {
                SearchCommand::Start { position, control } => self.search(position, control),
                SearchCommand::Stop => (),
                SearchCommand::Quit => break,
            }
        }
    }

    fn search(&mut self, position: Chess, control: SearchControl) {
        let start_time = std::time::Instant::now();

        // Determine sarch constraints
        let (max_depth, time_limit) = match control {
            SearchControl::ToDepth(depth) => (depth, u64::MAX),
            SearchControl::TimeLimit(time_limit) => (u8::MAX, time_limit),
        };

        // Check for external interrupts
        // match self.cmd_rx.try_recv() {
        //     Ok(SearchCommand::Start { .. }) | Ok(SearchCommand::Stop) => break 'outer,
        //     Ok(SearchCommand::Quit) => return,
        //     _ => (),
        // };
        //

        /////////
        let mut best_move = None;
        let mut best_score = i32::MIN + 1;

        for depth in 1..=max_depth {
            // Optional: reset per-iteration stats
            let mut iteration_best_move = None;
            let mut iteration_best_score = i32::MIN + 1;

            let mut moves = position.legal_moves();

            // 1️⃣ TT move ordering: try previous best move first
            let hash = position.zobrist_hash::<Zobrist64>(EnPassantMode::Legal);
            if let Some(tt_entry) = self.tt.lookup(hash) {
                if let Some(tt_best_move) = &tt_entry.best_move {
                    if let Some(i) = moves.iter().position(|m| m == tt_best_move) {
                        moves.swap(0, i);
                    }
                }
            }

            moves = order::order(moves);

            for mv in moves {
                // if stop_flag.load(Ordering::Relaxed) {
                //     break; // abort search
                // }

                let mut new_pos = position.clone();
                new_pos.play_unchecked(&mv);

                let score = -negamax(&new_pos, depth - 1, i32::MIN + 1, i32::MAX, 1, &mut self.tt);

                if score > iteration_best_score {
                    iteration_best_score = score;
                    iteration_best_move = Some(mv);
                }
            }

            // 2️⃣ Update global best from this iteration
            if let Some(iter_move) = iteration_best_move {
                best_move = Some(iter_move);
                best_score = iteration_best_score;
            }

            // 3️⃣ Extract PV from TT for reporting
            //let pv = extract_pv(board, tt);

            // 4️⃣ Optionally: send info (depth, score, nodes, pv) to UI

            // Stop early if the stop flag triggered
            // if stop_flag.load(Ordering::Relaxed) {
            //     break;
            // }

            self.info_tx
                .send(SearchInfo::Info {
                    depth,
                    pv: vec![best_move.clone().unwrap()],
                    score: best_score,
                    nodes: 1234,
                })
                .unwrap();

            // Check time left
            let elapsed = start_time.elapsed();
            if elapsed > std::time::Duration::from_millis(time_limit / 2) {
                break;
            }
        }

        let best_move = best_move.expect("No legal moves found");
        //let pv = extract_pv(board, tt);

        // Output best move
        self.info_tx.send(SearchInfo::BestMove(best_move)).unwrap();
    }
}
