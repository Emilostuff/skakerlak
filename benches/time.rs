use colour::*;
use crossbeam_channel::unbounded;
use skakarlak::search::Searcher;
use skakarlak::{SearchCommand, SearchControl, SearchInfo};
use std::io::Write;
use std::thread;
use std::time::Instant;

mod common;
use common::*;

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
    "3R2k1/2p5/6N1/p3p2P/3N1r2/6P1/3r4/b1K5 b - - 18 21",
    "4kb2/p1p5/3p2N1/1Pn1p2P/5r2/6P1/2nr3P/b1K5 w - - 18 10",
    "6QR/pk6/3p2N1/1P6/3N1r2/Kb4P1/3r4/2q2b2 w - - 11 7",
    "3Rkb2/p5P1/3p4/p1n4P/3N4/1b4P1/8/b4K2 b - - 8 3",
    "5bQR/p1k5/8/1Pn1p2P/3N1r2/1b4P1/2nr4/bK6 w - - 13 3",
    "2kR1bQ1/p1p3P1/3p2N1/pP2p3/3N4/1b3K2/2nr4/b1q5 b - - 26 21",
    "3R1bQR/2p5/k5N1/p1n1p2P/3N1r2/1b4P1/K1nr3P/2q5 w - - 39 27",
    "3R2QR/pkp5/3p2N1/p1n1p2P/8/1bK5/2nr3P/2q5 b - - 15 4",
    "7R/p2k2P1/3p4/p3p2P/8/1b4P1/2nr3P/b4bK1 b - - 27 3",
    "3R2k1/p5P1/3p2N1/pPn5/3N1r2/1b4P1/1Kn5/5b2 b - - 33 28",
    "3R3R/p1p5/6k1/1Pn1p3/8/1b4P1/2nr3P/b1q2bK1 w - - 27 25",
    "1k3bQ1/p1p5/8/pPn1p3/8/K5P1/2n5/b7 w - - 45 28",
    "7k/p1p5/6N1/1P2p3/8/1b6/2n4P/b1K2b2 b - - 11 1",
    "5bQR/k5P1/8/pP2p2P/3N1r2/1b3K2/3r3P/b1q2b2 w - - 40 4",
    "3R2Q1/p1p3P1/1k1p4/2n1p2P/3N4/1b4P1/2nr4/bK3b2 w - - 7 4",
    "5b1R/p1p3P1/3pk3/1Pn4P/8/1b6/1Kn4P/2q5 w - - 41 2",
    "5bQR/p1p3P1/3p2k1/pPn4P/3N1r2/8/2nK3P/b4b2 b - - 8 28",
    "1k1R1bQR/p5P1/6N1/pP5P/8/6P1/2n4P/5bK1 b - - 10 13",
    "2kR3R/p1p5/3p4/1P2p3/3N1r2/1K4P1/3r3P/b4b2 b - - 23 26",
    "3R2Q1/2p5/3p1kN1/1P2p2P/3N4/1b4P1/3r3P/b5K1 w - - 19 7",
    "3R1bQ1/p1k3P1/3p2N1/pP2p2P/3N1r2/1b1K4/2n4P/2q2b2 w - - 23 19",
    "3R3R/2p5/3pk1N1/1Pn1p2P/3N4/6K1/2nr3P/2q2b2 b - - 45 26",
    "3k1b1R/p1p5/3p4/1Pn1p2P/8/8/2nK3P/b1q2b2 w - - 14 26",
    "3R4/p1p3Pk/6N1/2n5/3N1r2/6P1/1Knr3P/b4b2 w - - 11 25",
    "3R2QR/2p3P1/2k3N1/p3p3/5r2/1b4P1/2nr3K/b4b2 w - - 23 8",
    "5bQ1/p1p3P1/1k4N1/pPn1p3/5r2/1b4P1/3r3P/K4b2 w - - 19 15",
];

fn search(time_limit: u64) -> Vec<Measurements> {
    let (cmd_tx, cmd_rx) = unbounded();
    let (info_tx, info_rx) = unbounded();

    thread::spawn(|| Searcher::new(cmd_rx, info_tx).run());

    white_bold!("\nSearching positions");
    black_ln!(" time limit: {} ms\n", time_limit);
    std::thread::sleep(std::time::Duration::from_millis(1000));

    let padding = pad(POSITIONS);

    POSITIONS
        .into_iter()
        .map(|fen| {
            magenta!("    {:padding$}   ", fen);
            black!("depth: ");

            let position = parse_fen(fen);

            let start = Instant::now();

            // send start signal
            cmd_tx
                .send(SearchCommand::Start {
                    position: position.clone(),
                    control: SearchControl::TimeLimit(time_limit),
                })
                .unwrap();

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
    let k_nodes_per_sec = nodes_per_sec
        .iter()
        .map(|n| *n as f64 / 1000.0)
        .collect::<Vec<_>>();

    // Print summary
    white_ln_bold!("\nSummary\n");
    plot_quantity("Depth", depths);
    plot_quantity("Throughput [kN/s]", k_nodes_per_sec);
}
