use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use shakmaty::{fen::Fen, uci::UciMove, CastlingMode, Chess, Position};
use shakmaty_uci::{UciInfo, UciInfoScore, UciMessage, UciTimeControl};
use skakarlak::UciInterface;
use std::error::Error;
use std::time::Duration;
use tokio::sync::{
    mpsc::{self, UnboundedSender},
    watch,
};
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut uci = UciInterface::new("engine.log")?;

    let (search_tx, mut search_rx) = mpsc::unbounded_channel::<UciMessage>();
    let (stop_tx, stop_rx) = watch::channel(false);

    let mut search_handle: Option<JoinHandle<()>> = None;
    let mut pos = Chess::default();

    loop {
        tokio::select! {
            // A message came from the search task. Send it to the GUI.
            Some(msg) = search_rx.recv() => {
                uci.send(&msg).await?;
            }

            // A command came from the GUI.
            received = uci.receive() => {
                match received? {
                    Some(UciMessage::Quit) | None => break,
                    Some(cmd) => {
                        handle_command(cmd, &mut uci, &mut pos, &mut search_handle, &search_tx, &stop_tx, &stop_rx,).await?
                    }
                }
            }
        }
    }

    // Before exiting, stop any running search and wait for it to finish.
    let _ = stop_tx.send(true);
    if let Some(handle) = search_handle.take() {
        handle.await?;
    }

    Ok(())
}

/// Processes a single UCI command, modifying state and sending responses.
/// Returns `true` if the engine should quit.
async fn handle_command(
    cmd: UciMessage,
    uci: &mut UciInterface,
    pos: &mut Chess,
    search_handle: &mut Option<JoinHandle<()>>,
    search_tx: &UnboundedSender<UciMessage>,
    stop_tx: &watch::Sender<bool>,
    stop_rx: &watch::Receiver<bool>,
) -> Result<(), Box<dyn Error>> {
    match cmd {
        UciMessage::Uci => {
            uci.send(&UciMessage::Id {
                name: Some(format!("Skakarlak {}", env!("CARGO_PKG_VERSION"))),
                author: None,
            })
            .await?;
            uci.send(&UciMessage::Id {
                name: None,
                author: Some("Emil Skydsgaard".into()),
            })
            .await?;
            uci.send(&UciMessage::UciOk).await?;
        }
        UciMessage::IsReady => {
            uci.send(&UciMessage::ReadyOk).await?;
        }
        UciMessage::UciNewGame | UciMessage::Position { .. } => {
            // Stop any ongoing search before changing the position.
            let _ = stop_tx.send(true);
            if let Some(handle) = search_handle.take() {
                handle.await?;
            }
            // If the command is Position, update the board.
            if let UciMessage::Position {
                startpos,
                fen,
                moves,
            } = cmd
            {
                *pos = make_position(startpos, fen, moves)?;
            } else {
                // Otherwise (UciNewGame), just reset to default.
                *pos = Chess::default();
            }
        }
        UciMessage::Go { time_control, .. } => {
            // Ensure previous search is stopped and awaited.
            let _ = stop_tx.send(true);
            if let Some(handle) = search_handle.take() {
                handle.await?;
            }

            let search_tx = search_tx.clone();
            let stop_rx = stop_rx.clone();
            let is_infinite = matches!(time_control, Some(UciTimeControl::Infinite));
            let _ = stop_tx.send(false); // Reset stop flag
            let search_pos = pos.clone();

            *search_handle = Some(tokio::spawn(async move {
                // Dummy search logic.

                let moves = search_pos.legal_moves();
                if moves.is_empty() {
                    return;
                }

                let mut seed = [0u8; 32];
                rand::rng().fill_bytes(&mut seed);
                let mut rng = ChaCha20Rng::from_seed(seed);
                let best_move = moves.choose(&mut rng).unwrap();

                let mut depth = 1;
                loop {
                    if *stop_rx.borrow() {
                        break;
                    }

                    let info_msg = UciMessage::Info(UciInfo {
                        depth: Some(depth),
                        score: Some(UciInfoScore {
                            cp: Some(10 * depth as i32),
                            mate: None,
                            lower_bound: false,
                            upper_bound: false,
                        }),
                        pv: vec![UciMove::from_move(best_move, CastlingMode::Standard)],
                        sel_depth: None,
                        time: None,
                        nodes: None,
                        multi_pv: None,
                        curr_move: None,
                        curr_move_num: None,
                        hash_full: None,
                        nps: None,
                        tb_hits: None,
                        sb_hits: None,
                        cpu_load: None,
                        string: None,
                        refutation: vec![],
                        curr_line: vec![],
                    });

                    if search_tx.send(info_msg).is_err() {
                        break;
                    }

                    tokio::time::sleep(Duration::from_millis(500)).await;
                    depth += 1;
                    if !is_infinite && depth > 5 {
                        break;
                    }
                }
                let _ = search_tx.send(UciMessage::BestMove {
                    best_move: UciMove::from_move(best_move, CastlingMode::Standard),
                    ponder: None,
                });
            }));
        }
        UciMessage::Stop => {
            let _ = stop_tx.send(true);
        }
        _ => {
            // Other commands are logged by `receive` but not handled here.
        }
    }
    Ok(())
}

/// Construct a `shakmaty::Chess` position from either `startpos` or a given FEN,
/// applying a sequence of UCI moves on top.
pub fn make_position(
    startpos: bool,
    fen: Option<Fen>,
    moves: Vec<UciMove>,
) -> Result<Chess, Box<dyn Error>> {
    let mut pos = if startpos {
        Chess::default()
    } else if let Some(fen) = fen {
        fen.into_position(CastlingMode::Standard)?
    } else {
        Chess::default()
    };

    for mv in moves {
        let m = mv.to_move(&pos)?;
        pos = pos.play(&m)?;
    }

    Ok(pos)
}
