mod common;
use common::*;

use colour::*;
use crossbeam_channel::unbounded;
use skakarlak::search::Searcher;
use skakarlak::{SearchCommand, SearchControl, SearchInfo};
use std::io::Write;
use std::thread;
use std::time::Instant;

const POSITIONS: &[&'static str] = &[
    "3k1b1R/p1p3P1/6N1/p3p2P/3N1r2/1b4P1/2nr3K/b1q2b2 w - - 39 16",
    "5bQR/p1k5/6N1/pPn4P/8/1K4P1/2n4P/b1q5 w - - 28 12",
    "1k1R2QR/p1p3P1/3p4/pP2p2P/3N4/1K6/2nr3P/5b2 b - - 42 8",
    "3R1bQ1/p1p5/3p1kN1/4p3/3N4/1b6/7K/b1q2b2 w - - 28 4",
    "6Q1/p1p3P1/4k3/pP2p2P/8/1b6/2nr4/2K5 b - - 23 1",
    "3k2QR/p1p5/8/1Pn4P/5r2/1b4P1/3K3P/b4b2 b - - 21 22",
    "8/p3k1P1/3p4/pPn5/5r2/1b4K1/2nr3P/b1q5 b - - 4 8",
    "3R2QR/2p5/5k2/p1n5/5r2/1b6/2nK3P/b1q2b2 w - - 34 13",
    "1k1R2QR/6P1/3p2N1/pP2p2P/3N4/1b6/7K/5b2 b - - 13 16",
    "5b1R/p1p5/3pk1N1/p6P/5r2/1bK3P1/2nr4/b4b2 w - - 37 8",
    "3R3k/2p5/3p4/1Pn4P/3N1r2/1b6/3r4/b1K5 b - - 3 16",
    "3R1bQR/p5P1/1k1p4/p1n4P/8/1b6/3r1K1P/b1q2b2 w - - 45 6",
    "3R1b1R/1k6/3p4/1Pn1p2P/8/1b4P1/3K4/2q5 w - - 14 26",
    "6Q1/2p3P1/3k2N1/1P2p3/3N4/8/3r3P/b1K2b2 b - - 35 23",
    "5bQR/p5P1/4k1N1/pPn4P/5r2/6PK/3r3P/b1q5 b - - 25 3",
    "3R1b2/pkp3P1/3p4/pPn1p2P/8/1b5K/2nr3P/8 w - - 5 26",
    "5b2/p1p3k1/3p2N1/p1n1p2P/3N4/1b4P1/8/3K1b2 w - - 27 14",
    "7R/2pk2P1/3p4/p1n1p2P/3N1r2/1b6/2nr4/b1qK4 w - - 12 25",
    "3k1bQ1/p1p3P1/8/2n1p2P/5r2/1b4P1/2nrK2P/5b2 w - - 39 2",
    "3R1bQR/p1k3P1/3p4/pPn1p3/3N4/6PK/2n4P/b1q5 b - - 35 17",
    "5b1R/p5P1/1k1p4/1Pn5/3N1r2/1b6/3r3P/b3Kb2 w - - 6 4",
    "3R1bQR/p1k3P1/3p2N1/pPn4P/3N1r2/8/3rK3/b1q5 w - - 16 29",
    "3R4/p1p3P1/2k3N1/pPn1p3/5r2/1b6/3r3P/bK6 b - - 7 5",
    "3R2QR/k5P1/8/p1n4P/5r2/1b4P1/3r3K/2q5 w - - 41 18",
    "3R1bQ1/6P1/1k1p2N1/p1n1p3/3N4/1b4P1/2n5/5bK1 b - - 9 17",
    "5bQ1/pk4P1/3p4/7P/5r2/5KP1/2nr3P/b1q5 w - - 22 7",
    "6Q1/k1p3P1/3p2N1/1Pn1p3/5r2/1b6/2n3K1/b1q2b2 w - - 1 22",
    "3R1bQR/2p3P1/5k2/p7/5r2/1b6/2nK3P/b4b2 w - - 13 19",
    "5bQR/2k3P1/3p2N1/p7/3N4/1b4P1/2nr3P/b2K1b2 w - - 0 18",
    "3R1b1R/2p5/1k1p2N1/1Pn5/3N1r2/1K6/2n5/b1q2b2 w - - 24 14",
    "3k1bQR/p1p3P1/6N1/2n1p2P/3N1r2/6P1/2Kr3P/8 w - - 27 25",
    "3R1bQR/p1k3P1/3p4/p1n4P/3N4/3K4/2n5/b4b2 w - - 40 28",
    "3R1bQ1/2p3P1/1k4N1/pPn1p2P/3N1r2/6P1/2n4K/2q2b2 w - - 21 14",
    "5bQR/1kp3P1/6N1/pP6/3N1r2/1K6/7P/2q2b2 b - - 43 28",
    "6QR/p1p1k1P1/3p4/2n4P/5r2/1b6/2n4P/b1qK4 w - - 33 18",
    "4kbQ1/p5P1/8/pP2p3/3N4/1bK3P1/2nr3P/5b2 b - - 13 5",
    "1k3b1R/p7/8/pP2p3/3N4/1b3K2/2nr3P/b1q2b2 w - - 9 7",
    "3R2Q1/pk4P1/3p2N1/p3p2P/3N4/5K2/7P/2q2b2 b - - 44 30",
    "5bQR/pk4P1/3p4/2n1p2P/8/6P1/2n4P/K4b2 w - - 30 22",
    "5bQ1/p7/3p1k2/p6P/3N4/1b6/2n4K/b1q2b2 b - - 0 26",
    "6QR/2p3P1/3k4/pPn1p3/5r2/6P1/2nr3P/b2K1b2 w - - 21 14",
    "5bQR/2k3P1/3p4/1P2p3/3N1r2/1b6/2n4P/K7 w - - 39 1",
    "3R1bQ1/p1p4k/3p2N1/1Pn1p3/3N1r2/1b4P1/4K2P/b7 b - - 5 22",
    "5bQk/2p3P1/3p4/p1n1p3/3N1r2/6P1/2nr4/2K2b2 b - - 27 23",
    "5bQ1/3k2P1/3p2N1/pPn1p3/3N4/1K6/2nr3P/b1q5 w - - 37 12",
    "3R3R/8/3pk1N1/pPn1p2P/5r2/1K4P1/2nr4/b7 w - - 33 10",
    "2kR1b1R/p1p3P1/6N1/1Pn1p3/3N1r2/1b6/2nr4/bK6 b - - 12 26",
    "3R1bk1/p7/6N1/1P2p2P/3N1r2/6P1/2nr3K/b1q2b2 w - - 32 27",
    "3R1bk1/p1p5/3p2N1/pP2p2P/3N1r2/1b4P1/2n4P/b1K5 w - - 3 30",
    "6QR/2k3P1/3p4/pPn1p3/3N4/4K1P1/7P/b7 w - - 27 28",
    "1k1R1bQ1/2p3P1/6N1/p6P/3N1r2/6P1/2nK3P/b7 b - - 26 21",
    "3R1b2/2p3k1/8/2n4P/3N1r2/Kb4P1/7P/b1q2b2 w - - 24 1",
    "5bQk/p7/6N1/pPn4P/3N4/6P1/3r3P/b1K5 b - - 6 30",
    "3k4/p1p5/3p4/pP2p2P/3N4/1bK3P1/3r3P/8 w - - 34 19",
];

fn search(depth: u8) -> Vec<Measurements> {
    let (cmd_tx, cmd_rx) = unbounded();
    let (info_tx, info_rx) = unbounded();

    thread::spawn(|| Searcher::new(cmd_rx, info_tx).run());

    white_bold!("\nSearching positions");
    black_ln!(" to depth: {}\n", depth);
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
                    control: SearchControl::ToDepth(depth),
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
                        elapsed = new_elapsed;
                        nodes += n;
                        depth = d;

                        white!("{} ", d);
                        std::io::stdout().flush().unwrap();
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
    let results = search(7);

    // Extract values
    let mut nodes = results.iter().map(|m| m.nodes).collect::<Vec<_>>();
    nodes.sort();
    let mut times = results.iter().map(|m| m.time_ms).collect::<Vec<_>>();
    times.sort();
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
    plot_quantity("Nodes", nodes);
    plot_quantity("Time [ms]", times);
    plot_quantity("Throughput [kN/s]", k_nodes_per_sec);
}
