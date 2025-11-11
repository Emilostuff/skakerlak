use chrono::Local;
use shakmaty_uci::UciMessage;
use std::fs::OpenOptions;
use std::io::Write;
use tokio::io::{stdin, stdout, AsyncBufReadExt, AsyncWriteExt, BufReader, Stdin, Stdout};

pub struct UciInterface {
    input: BufReader<Stdin>,
    output: Stdout,
    log_path: &'static str,
}

impl UciInterface {
    /// Creates a new UciInterface.
    /// This will open the log file and log the startup message.
    pub fn new(log_path: &'static str) -> std::io::Result<Self> {
        let input = BufReader::new(stdin());
        let output = stdout();

        let mut instance = Self {
            input,
            output,
            log_path,
        };

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        instance.log("")?; // Add a blank line for readability
        instance.log(&format!("------ Engine started at {} ------", timestamp))?;

        Ok(instance)
    }

    /// Logs a line to the log file held by the UciInterface.
    fn log(&mut self, line: &str) -> std::io::Result<()> {
        let mut log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.log_path)?;
        // Panicking on a logging error is reasonable for this application.
        writeln!(&mut log_file, "{}", line)
    }

    /// Sends a UCI message to stdout, ensuring it's flushed.
    pub async fn send(&mut self, msg: &UciMessage) -> std::io::Result<()> {
        // Format the message and add the required newline for the UCI protocol.
        let msg_string = format!("{}\n", msg);
        let reordered_msg = Self::reorder_uci_info(&msg_string);

        // Log the outgoing message, trimming the newline for a cleaner log.
        self.log(&format!("OUT: '{}'", msg_string.trim()))?;
        self.log(&format!("OUT actual: '{}'", reordered_msg.trim()))?;

        self.output.write_all(reordered_msg.as_bytes()).await?;
        self.output.flush().await?;

        Ok(())
    }

    fn reorder_uci_info(line: &str) -> String {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() != 8 || parts[0] != "info" {
            return line.to_string();
        }

        // info depth 6 pv e7e6 score cp -1222
        // -> info depth 6 score cp -1222 pv e7e6
        format!(
            "{} {} {} {} {} {} {} {}\n",
            parts[0], // info
            parts[1], // depth
            parts[2], // 6
            parts[5], // score
            parts[6], // cp
            parts[7], // -1222
            parts[3], // pv
            parts[4]  // e7e6
        )
    }

    pub async fn receive(&mut self) -> std::io::Result<Option<UciMessage>> {
        loop {
            let mut line = String::new();
            let bytes_read = self.input.read_line(&mut line).await?;

            if bytes_read == 0 {
                return Ok(None); // End of stream
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                // Ignore empty lines and wait for the next command.
                continue;
            }

            match trimmed.parse::<UciMessage>() {
                Ok(msg) => {
                    // Log the successfully parsed message.
                    self.log(&format!(" IN: '{}'", trimmed))?;
                    return Ok(Some(msg));
                }
                Err(e) => {
                    // A parsing error is not critical. Log the failure and continue.
                    self.log(&format!(" IN: '{}' --- {}!", trimmed, e))?;
                }
            }
        }
    }
}

impl Drop for UciInterface {
    fn drop(&mut self) {
        self.log("------ Engine closed ------").unwrap();
    }
}
