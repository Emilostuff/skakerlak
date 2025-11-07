use std::env;
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    // Path to the release binary
    let exe_name = "skakarlak";
    let exe_path = format!("./{}", exe_name);

    // Pass through any arguments given to runner
    let args: Vec<String> = env::args().skip(1).collect();

    // Spawn the child process
    let child = match Command::new(&exe_path)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            eprintln!("Failed to start main binary '{}': {}", exe_path, e);
            std::process::exit(1);
        }
    };

    // Wrap child in Arc<Mutex<Child>> for safe access
    let child_arc = Arc::new(Mutex::new(child));

    // Take child stdin and stdout
    let child_stdin = child_arc
        .lock()
        .unwrap()
        .stdin
        .take()
        .expect("Failed to open child stdin");
    let child_stdout = child_arc
        .lock()
        .unwrap()
        .stdout
        .take()
        .expect("Failed to open child stdout");

    let child_stdin_arc = Arc::new(Mutex::new(child_stdin));
    let child_stdout_arc = Arc::new(Mutex::new(child_stdout));

    // Channel to signal quit
    let (quit_tx, quit_rx) = std::sync::mpsc::channel();

    // Thread to forward stdin to child, and detect "quit"
    {
        let quit_tx = quit_tx.clone();
        let child_stdin_arc = Arc::clone(&child_stdin_arc);
        thread::spawn(move || {
            let stdin = io::stdin();
            let mut handle = stdin.lock();
            let mut buffer = [0u8; 4096];
            loop {
                match handle.read(&mut buffer) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let input_str = String::from_utf8_lossy(&buffer[..n]).to_lowercase();
                        if input_str.contains("quit") {
                            let _ = quit_tx.send(());
                            // Forward "quit" to child before killing
                            let _ = child_stdin_arc.lock().unwrap().write_all(&buffer[..n]);
                            break;
                        }
                        if child_stdin_arc
                            .lock()
                            .unwrap()
                            .write_all(&buffer[..n])
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }

    // Thread to forward child stdout to our stdout
    {
        let child_stdout_arc = Arc::clone(&child_stdout_arc);
        thread::spawn(move || {
            let stdout = io::stdout();
            let mut stdout_handle = stdout.lock();
            let mut buffer = [0u8; 4096];
            loop {
                match child_stdout_arc.lock().unwrap().read(&mut buffer) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        if stdout_handle.write_all(&buffer[..n]).is_err() {
                            break;
                        }
                        stdout_handle.flush().ok();
                    }
                    Err(_) => break,
                }
            }
        });
    }

    // Main thread: wait for quit or child exit
    let status = loop {
        // If quit signal received, kill child
        if let Ok(_) = quit_rx.try_recv() {
            let mut child = child_arc.lock().unwrap();
            let _ = child.kill();
            let status = child.wait().expect("Failed to wait on child process");
            break status;
        }
        // Check if child has exited
        let mut child = child_arc.lock().unwrap();
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                drop(child); // Release lock before sleeping
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                eprintln!("Error waiting for child: {}", e);
                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    break std::process::ExitStatus::from_raw(1);
                }
                #[cfg(not(unix))]
                break std::process::ExitStatus::from_raw(1);
            }
        }
    };

    std::process::exit(status.code().unwrap_or(1));
}
