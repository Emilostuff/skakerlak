use colour::*;
use crossbeam_channel::unbounded;
use shakmaty::fen::Fen;
use shakmaty::{CastlingMode, Chess};
use skakarlak::search::Searcher;
use skakarlak::{SearchCommand, SearchControl, SearchInfo};
use std::io::Write;
use std::str::FromStr;
use std::thread;
use std::time::Instant;

const POSITIONS: &[&'static str] = &[
    "6Q1/p1p3P1/1k1p2N1/p1n1p2P/5r2/1b6/2n4K/b1q2b2 b - - 29 30",
    "6QR/8/3p1kN1/1P5P/3N1r2/1b4P1/3r4/2K2b2 b - - 13 10",
    "5b2/2pk2P1/3p2N1/pP2p2P/5r2/1b4P1/2K5/b1q5 w - - 17 16",
    "3R3R/2p3P1/1k1p4/pP6/3N4/1b4P1/2nr4/b3K3 w - - 34 9",
    "3k2QR/p5P1/3p2N1/pPn1p3/5r2/1b4P1/2n4P/b1K5 b - - 2 25",
    "6QR/2p3P1/6k1/pPn1p2P/3N1r2/8/K1n5/b1q2b2 b - - 21 2",
    "4k1Q1/p5P1/6N1/pPn1p3/8/1b4P1/3r3P/2K2b2 b - - 26 11",
    "3R1b2/k1p3P1/6N1/pPn4P/5r2/6K1/2n4P/b1q5 w - - 13 23",
    "5bQ1/k1p3P1/8/1P5P/3N1r2/1b4P1/K1nr3P/b1q2b2 w - - 22 6",
    "4kbQR/p5P1/8/p7/3N4/Kb4P1/3r4/2q2b2 w - - 6 18",
    "3R1bQ1/k5P1/3p2N1/pP2p2P/3N4/8/2n4P/1Kq5 w - - 11 1",
    "3R1bQR/k1p5/3p4/pPn1p3/8/1b4P1/2K4P/b4b2 w - - 8 11",
    "3R2QR/pk4P1/3p2N1/1Pn1p3/5r2/1b1K4/3r3P/2q5 w - - 6 30",
    "4kb1R/2p3P1/3p4/p1n1p3/3N1r2/8/2K4P/b4b2 w - - 4 12",
    "3R1b1k/p7/8/4p2P/3N1r2/1b2K1P1/2nr3P/5b2 w - - 12 22",
    "3R2Q1/1kp5/3p2N1/pPn1p2P/3N4/6P1/8/b4K2 w - - 15 15",
    "3R1bQ1/k5P1/3p2N1/p1n1p2P/8/1b6/1Knr3P/2q2b2 w - - 26 3",
    "3R1bQR/2p5/3p1kN1/pPn1p3/5r2/6P1/3r2KP/b4b2 w - - 15 5",
    "5b2/pkp3P1/3p4/7P/8/1b4K1/2nr3P/b1q2b2 w - - 39 11",
    "3R2Q1/p1p1k3/8/p3p2P/5r2/6PK/8/b1q2b2 w - - 10 6",
];

fn parse_fen(fen_str: &str) -> Chess {
    Fen::from_str(fen_str)
        .unwrap()
        .into_position(CastlingMode::Standard)
        .unwrap()
}

fn pad() -> usize {
    POSITIONS.iter().map(|x| x.len()).max().unwrap()
}

struct Measurements {
    depth: u8,
    time_ms: u128,
    nodes: u64,
}

fn search(time_limit: u64) -> Vec<Measurements> {
    let (cmd_tx, cmd_rx) = unbounded();
    let (info_tx, info_rx) = unbounded();

    thread::spawn(|| Searcher::new(cmd_rx, info_tx).run());

    white_bold!("\nSearching positions");
    black_ln!(" ≤ {} ms\n", time_limit);
    let padding = pad();

    POSITIONS
        .into_iter()
        .map(|fen| {
            magenta!("    {:padding$}   ", fen);
            black!("depth: ");

            let position = parse_fen(fen);

            // send start signal
            cmd_tx
                .send(SearchCommand::Start {
                    position: position.clone(),
                    control: SearchControl::TimeLimit(time_limit),
                })
                .unwrap();

            // start time
            let start = Instant::now();
            let mut elapsed = 0;
            let mut nodes = 0;
            let mut depth = 0;

            // Wait for best move output
            loop {
                match info_rx.recv() {
                    Ok(SearchInfo::BestMove(_)) => {
                        println!();
                        break;
                    }
                    Ok(SearchInfo::Info {
                        nodes: n, depth: d, ..
                    }) => {
                        let new_elapsed = start.elapsed().as_millis();
                        if new_elapsed <= time_limit as u128 {
                            elapsed = new_elapsed;
                            nodes += n;
                            depth = d;

                            white!("{} ", d);
                            std::io::stdout().flush().unwrap();
                        }
                    }
                    _ => (),
                }
            }

            // Clear TT and other cached values
            cmd_tx.send(SearchCommand::Reset).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(50));

            Measurements {
                depth,
                time_ms: elapsed,
                nodes,
            }
        })
        .collect()
}

fn main() {
    let results = search(200);

    // Extract values
    let mut depths = results.iter().map(|m| m.depth as usize).collect::<Vec<_>>();
    depths.sort();

    let mut nodes_per_sec = results
        .iter()
        .map(|m| (m.nodes as f64 / m.time_ms as f64 * 1000.0) as u64)
        .collect::<Vec<_>>();
    nodes_per_sec.sort();

    // Compute statistics
    let min_depth = depths[0];
    let median_depth = depths[depths.len() / 2];
    let max_depth = depths[depths.len() - 1];
    let min_k_nodes_per_sec = nodes_per_sec[0] as f64 / 1000.0;
    let median_k_nodes_per_sec = nodes_per_sec[nodes_per_sec.len() / 2] as f64 / 1000.0;
    let max_k_nodes_per_sec = nodes_per_sec[nodes_per_sec.len() - 1] as f64 / 1000.0;

    // Print summary
    white_ln_bold!("\nSummary\n");
    blue_ln_bold!("    Depth");
    black!("    Min: ");
    yellow!("{}", min_depth);
    black!("   Median: ");
    white_bold!("{}", median_depth);
    black!("   Max: ");
    green_ln!("{}", max_depth);
    println!();

    blue_ln_bold!("    Throughput [kN/s] ");
    black!("    Min: ");
    yellow!("{:.1}", min_k_nodes_per_sec);
    black!("   Median: ");
    white_bold!("{:.1}", median_k_nodes_per_sec);
    black!("   Max: ");
    green_ln!("{:.1}", max_k_nodes_per_sec);
    println!();
}
