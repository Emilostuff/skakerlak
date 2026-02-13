use crossbeam_channel::Sender;
use shakmaty_uci::UciMessage;
use std::io::{self, BufRead};

/// Listens for UCI commands on stdin and forwards them to the input channel.
pub struct InputListener {
    input_tx: Sender<UciMessage>,
}

impl InputListener {
    pub fn new(input_tx: Sender<UciMessage>) -> Self {
        Self { input_tx }
    }

    pub fn run(self) {
        let stdin = io::stdin();

        // Listen while stdin is open
        for line_result in stdin.lock().lines() {
            let line = if let Ok(l) = line_result { l } else { continue };

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Try to parse as UciMessage. Ignore invalid input.
            // if let Ok(msg) = trimmed.parse::<UciMessage>() {
            //     self.input_tx.send(msg).unwrap();
            // }
            match trimmed.parse::<UciMessage>() {
                Ok(msg) => self.input_tx.send(msg).unwrap(),
                Err(e) => println!("Invalid input: {}", e),
            }
        }
    }
}
