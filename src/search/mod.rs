pub mod negamax;

use negamax::negamax;

use crate::{SearchCommand, SearchControl, SearchInfo};
use crossbeam_channel::{Receiver, Sender};
use shakmaty::{Chess, Move, Position};

pub struct Searcher {
    cmd_rx: Receiver<SearchCommand>,
    info_tx: Sender<SearchInfo>,
}

struct MoveCandidate {
    mv: Move,
    next_position: Chess,
}

impl Searcher {
    pub fn new(cmd_rx: Receiver<SearchCommand>, info_tx: Sender<SearchInfo>) -> Self {
        Searcher { cmd_rx, info_tx }
    }

    pub fn run(self) {
        loop {
            match self.cmd_rx.recv().unwrap() {
                SearchCommand::Start { position, control } => self.search(position, control),
                SearchCommand::Stop => (),
                SearchCommand::Quit => break,
            }
        }
    }

    fn search(&self, position: Chess, control: SearchControl) {
        let start_time = std::time::Instant::now();

        // Determine sarch constraints
        let (depth, time_limit) = match control {
            SearchControl::ToDepth(depth) => (depth, u64::MAX),
            SearchControl::TimeLimit(time_limit) => (u8::MAX, time_limit),
        };

        let mut moves: Vec<_> = position
            .legal_moves()
            .iter()
            .map(|m| {
                let mut pos = position.clone();
                pos.play_unchecked(m);
                (
                    MoveCandidate {
                        mv: m.clone(),
                        next_position: pos,
                    },
                    0,
                )
            })
            .collect();

        let mut pv = Vec::new();

        'outer: for d in 0..=depth - 1 {
            let mut nodes = 0;
            let mut best_pv = Vec::new();
            let mut best_score = i32::MIN + 1;

            for (move_candidate, score) in &mut moves {
                // Check for external interrupts
                match self.cmd_rx.try_recv() {
                    Ok(SearchCommand::Start { .. }) | Ok(SearchCommand::Stop) => break 'outer,
                    Ok(SearchCommand::Quit) => return,
                    _ => (),
                };

                // Check time left
                let elapsed = start_time.elapsed();
                if elapsed > std::time::Duration::from_millis((time_limit as f32 * 0.9) as u64) {
                    break 'outer;
                }

                // Use pv from previous iteration to guide search (if it starts with the current move)
                let pv_slice = if let Some(mv) = pv.first() {
                    if mv == &move_candidate.mv {
                        &[]
                    } else {
                        &[]
                    }
                } else {
                    &[]
                };

                let (opponents_score, new_pv) = negamax(
                    &move_candidate.next_position,
                    d,
                    -i32::MAX,
                    -best_score,
                    0,
                    &mut nodes,
                    pv_slice,
                );

                *score = -opponents_score;

                if *score > best_score {
                    best_score = *score;
                    best_pv = vec![move_candidate.mv.clone()];
                    best_pv.extend_from_slice(&new_pv);
                }
            }

            moves.sort_by_key(|(_, score)| -*score);
            pv = best_pv;

            self.info_tx
                .send(SearchInfo::Info {
                    depth: d + 1,
                    pv: pv.clone(),
                    score: best_score,
                    nodes,
                })
                .unwrap();

            // if check found at this depth, don't search further
            if best_score >= i32::MAX - depth as i32 {
                break;
            }

            // Check time left
            let elapsed = start_time.elapsed();
            if elapsed > std::time::Duration::from_millis(time_limit / 2) {
                break;
            }
        }

        // Output best move
        self.info_tx
            .send(SearchInfo::BestMove(pv[0].clone()))
            .unwrap();
    }
}
