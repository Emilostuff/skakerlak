use std::fs::OpenOptions;
use std::io::{self, BufRead, Write};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;

fn log_line(line: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("engine.log")
        .unwrap();
    writeln!(file, "{}", line).unwrap();
}

fn main() {
    let stdin = io::stdin();
    let stdout = Arc::new(Mutex::new(io::stdout()));
    let stop_flag = Arc::new(AtomicBool::new(false));
    let mut position_seen = false;

    log_line("Engine started");

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        // Trim and normalize command
        let cmd = line.replace("\r", "").trim().to_lowercase();
        log_line(&format!("IN: {:?}", line));

        match cmd.as_str() {
            "uci" => {
                let mut out = stdout.lock().unwrap();
                writeln!(out, "id name MyRustBot").unwrap();
                writeln!(out, "id author Emil Skydsgaard").unwrap();
                writeln!(out, "id version {}", env!("CARGO_PKG_VERSION")).unwrap();
                writeln!(out, "uciok").unwrap();
                out.flush().unwrap();
                log_line("OUT: uciok sent");
            }

            "isready" => {
                let mut out = stdout.lock().unwrap();
                writeln!(out, "readyok").unwrap();
                out.flush().unwrap();
                log_line("OUT: readyok sent");
            }

            "ucinewgame" => {
                position_seen = false;
                stop_flag.store(true, Ordering::SeqCst);
                log_line("New game started");
            }

            cmd if cmd.starts_with("position") => {
                position_seen = true;
                log_line(&format!("Got position: {:?}", line));
            }

            cmd if cmd.starts_with("go") => {
                if !position_seen {
                    log_line("Go received before position, ignoring");
                    continue;
                }

                let stop_clone = stop_flag.clone();
                stop_flag.store(false, Ordering::SeqCst);
                let stdout_clone = stdout.clone();
                let is_infinite = cmd.contains("infinite");

                if is_infinite {
                    // Infinite thinking thread
                    thread::spawn(move || {
                        let mut depth = 1;
                        while !stop_clone.load(Ordering::SeqCst) {
                            {
                                let mut out = stdout_clone.lock().unwrap();
                                writeln!(
                                    out,
                                    "info depth {} score cp {} pv e2e4",
                                    depth,
                                    10 * depth
                                )
                                .unwrap();
                                out.flush().unwrap();
                            }
                            thread::sleep(Duration::from_millis(500));
                            depth += 1;
                        }
                        {
                            let mut out = stdout_clone.lock().unwrap();
                            writeln!(out, "bestmove e2e4").unwrap();
                            out.flush().unwrap();
                        }
                        log_line("OUT: bestmove e2e4 sent");
                    });
                } else {
                    // Normal go: simulate thinking for a few steps
                    thread::spawn(move || {
                        let mut depth = 1;
                        while depth <= 10 {
                            {
                                let mut out = stdout_clone.lock().unwrap();
                                writeln!(
                                    out,
                                    "info depth {} score cp {} pv e2e4",
                                    depth,
                                    10 * depth
                                )
                                .unwrap();
                                out.flush().unwrap();
                            }
                            thread::sleep(Duration::from_millis(500));
                            depth += 1;
                        }
                        {
                            let mut out = stdout_clone.lock().unwrap();
                            writeln!(out, "bestmove e2e4").unwrap();
                            out.flush().unwrap();
                        }
                        log_line("OUT: bestmove e2e4 sent");
                    });
                }
            }

            "stop" => {
                stop_flag.store(true, Ordering::SeqCst);
                log_line("Received stop");
            }

            "quit" => {
                stop_flag.store(true, Ordering::SeqCst);
                log_line("Received quit");
                break;
            }

            _ => log_line(&format!("Unknown command: {:?}", line)),
        }
    }

    log_line("Engine exited");
}
