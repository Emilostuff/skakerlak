use crossbeam_channel::Sender;
use shakmaty_uci::UciMessage;
use std::io::{self, BufRead};

pub struct InputListener {
    input_tx: Sender<UciMessage>,
}

impl InputListener {
    pub fn new(input_tx: Sender<UciMessage>) -> Self {
        Self { input_tx }
    }

    pub fn run(self) {
        let stdin = io::stdin();
        for line_result in stdin.lock().lines() {
            let line = match line_result {
                Ok(l) => l,
                Err(_) => continue, // skip lines that can't be read
            };

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue; // skip empty lines
            }

            // Try to parse as UciMessage, ignore parse errors
            if let Ok(msg) = trimmed.parse::<UciMessage>() {
                self.input_tx.send(msg).unwrap();
            }
        }
    }
}
