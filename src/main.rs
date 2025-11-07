use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use shakmaty;
use shakmaty_uci::{UciMessage, UciTimeControl};
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;
use tokio::io::{stdin, stdout, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;

fn log_line(line: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("engine.log")
        .unwrap();
    writeln!(file, "{}", line).unwrap();
}

#[tokio::main]
async fn main() {
    let mut reader = BufReader::new(stdin()).lines();

    // Channel for search task to send info/bestmove to main task
    let (search_tx, mut search_rx) = mpsc::unbounded_channel::<String>();
    // Watch channel to signal stop to the search task
    let (stop_tx, stop_rx) = watch::channel(false);

    log_line("");
    log_line("Engine started");

    // Task to forward search info/bestmove to stdout
    tokio::spawn(async move {
        let mut out = stdout();
        while let Some(msg) = search_rx.recv().await {
            out.write_all(msg.as_bytes()).await.unwrap();
            out.flush().await.unwrap();
        }
    });

    let mut search_handle: Option<JoinHandle<()>> = None;

    // Position
    let mut pos = Chess::default();

    while let Some(line) = reader.next_line().await.unwrap_or(None) {
        let cmd = line.trim();
        log_line(&format!("IN: {:?}", cmd));

        match cmd.parse() {
            Ok(UciMessage::Uci) => {
                let mut out = stdout();
                out.write_all(
                    format!("id name Skakarlak {}\n", env!("CARGO_PKG_VERSION")).as_bytes(),
                )
                .await
                .unwrap();
                out.write_all(b"id author Emil Skydsgaard\n").await.unwrap();
                out.write_all(b"uciok\n").await.unwrap();
                out.flush().await.unwrap();
                log_line("OUT: uciok sent");
            }
            Ok(UciMessage::IsReady) => {
                let mut out = stdout();
                out.write_all(b"readyok\n").await.unwrap();
                out.flush().await.unwrap();
                log_line("OUT: readyok sent");
            }
            Ok(UciMessage::UciNewGame) => {
                let _ = stop_tx.send(true);
                log_line("New game started");
            }
            Ok(UciMessage::Position {
                startpos,
                fen,
                moves,
            }) => {
                pos = if startpos {
                    Chess::default()
                } else {
                    make_position(startpos, fen, moves).unwrap()
                };
            }
            Ok(UciMessage::Go {
                time_control,
                search_control: _,
            }) => {
                // Stop any previous search
                let _ = stop_tx.send(true);

                // Prepare for new search
                let search_tx = search_tx.clone();
                let stop_rx = stop_rx.clone();
                let is_infinite = matches!(time_control, Some(UciTimeControl::Infinite));

                // Reset stop flag
                let _ = stop_tx.send(false);

                let pos = pos.clone();

                // Spawn search task
                search_handle = Some(tokio::spawn(async move {
                    let moves = pos.legal_moves();
                    let mut seed = [0u8; 32];
                    rand::rng().fill_bytes(&mut seed);
                    let mut rng = ChaCha20Rng::from_seed(seed);
                    let best_move = moves
                        .as_slice()
                        .choose(&mut rng)
                        .map(|m| m.to_uci(CastlingMode::Standard))
                        .unwrap();
                    let mut depth = 1;
                    loop {
                        if *stop_rx.borrow() {
                            break;
                        }
                        let info = format!(
                            "info depth {} score cp {} pv {}\n",
                            depth,
                            10 * depth,
                            best_move
                        );
                        let _ = search_tx.send(info);
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        depth += 1;
                        if !is_infinite && depth > 5 {
                            break;
                        }
                    }
                    let _ = search_tx.send(format!("bestmove {}\n", best_move));
                    log_line(&format!("OUT: bestmove {} sent", best_move));
                }));
            }
            Ok(UciMessage::Stop) => {
                let _ = stop_tx.send(true);
                log_line("Received stop");
            }
            Ok(UciMessage::Quit) => {
                let _ = stop_tx.send(true);
                log_line("Received quit");
                break;
            }
            Ok(cmd) => {
                log_line(&format!("Received: {:?}, but did not handle it.", cmd));
            }
            Err(e) => log_line(&format!("Error parsing UCI command: {}", e)),
        }
    }

    // Wait for search task to finish if running
    if let Some(handle) = search_handle {
        let _ = handle.await;
    }

    log_line("Engine exited");
}

use shakmaty::{fen::Fen, uci::UciMove, CastlingMode, Chess, Position};

/// Construct a `shakmaty::Chess` position from either `startpos` or a given FEN,
/// applying a sequence of UCI moves on top.
///
/// # Arguments
///
/// * `startpos` — whether to start from the standard initial position.
/// * `fen` — optional FEN; ignored if `startpos` is `true`.
/// * `moves` — list of UCI moves to apply.
///
/// # Returns
///
/// The resulting `shakmaty::Chess` position after applying all moves.
pub fn make_position(
    startpos: bool,
    fen: Option<Fen>,
    moves: Vec<UciMove>,
) -> Result<Chess, Box<dyn std::error::Error>> {
    // 1. Choose starting position
    let mut pos = if startpos {
        Chess::default()
    } else if let Some(fen) = fen {
        fen.into_position(CastlingMode::Standard)?
    } else {
        // Fallback: no startpos and no FEN means default start
        Chess::default()
    };

    // 2. Apply all moves sequentially
    for mv in moves {
        let m = mv.to_move(&pos)?;
        pos = pos.play(&m)?;
    }

    Ok(pos)
}
