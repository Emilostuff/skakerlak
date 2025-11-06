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

    while let Some(line) = reader.next_line().await.unwrap_or(None) {
        let cmd = line.replace("\r", "").trim().to_lowercase();
        log_line(&format!("IN: {:?}", line));

        match cmd.as_str() {
            "uci" => {
                let mut out = stdout();
                out.write_all(b"id name MyRustBot\n").await.unwrap();
                out.write_all(b"id author Emil Skydsgaard\n").await.unwrap();
                out.write_all(format!("id version {}\n", env!("CARGO_PKG_VERSION")).as_bytes())
                    .await
                    .unwrap();
                out.write_all(b"uciok\n").await.unwrap();
                out.flush().await.unwrap();
                log_line("OUT: uciok sent");
            }
            "isready" => {
                let mut out = stdout();
                out.write_all(b"readyok\n").await.unwrap();
                out.flush().await.unwrap();
                log_line("OUT: readyok sent");
            }
            "ucinewgame" => {
                let _ = stop_tx.send(true);
                log_line("New game started");
            }
            cmd if cmd.starts_with("position") => {
                log_line(&format!("Got position: {:?}", line));
            }
            cmd if cmd.starts_with("go") => {
                // Stop any previous search
                let _ = stop_tx.send(true);

                // Prepare for new search
                let search_tx = search_tx.clone();
                let stop_rx = stop_rx.clone();
                let is_infinite = cmd.contains("infinite");

                // Reset stop flag
                let _ = stop_tx.send(false);

                // Spawn search task
                search_handle = Some(tokio::spawn(async move {
                    let mut depth = 1;
                    loop {
                        if *stop_rx.borrow() {
                            break;
                        }
                        let info =
                            format!("info depth {} score cp {} pv e2e4\n", depth, 10 * depth);
                        let _ = search_tx.send(info);
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        depth += 1;
                        if !is_infinite && depth > 10 {
                            break;
                        }
                    }
                    let _ = search_tx.send("bestmove e2e4\n".to_string());
                    log_line("OUT: bestmove e2e4 sent");
                }));
            }
            "stop" => {
                let _ = stop_tx.send(true);
                log_line("Received stop");
            }
            "quit" => {
                let _ = stop_tx.send(true);
                log_line("Received quit");
                break;
            }
            _ => log_line(&format!("Unknown command: {:?}", line)),
        }
    }

    // Wait for search task to finish if running
    if let Some(handle) = search_handle {
        let _ = handle.await;
    }

    log_line("Engine exited");
}
