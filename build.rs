use shakmaty::fen::Fen;
use shakmaty::{CastlingMode, Chess};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::str::FromStr;

fn main() {
    // Path to the FENs file
    let fens_path = "assets/fens.txt";

    // Instruct Cargo to rerun build.rs if the FENs file changes
    println!("cargo:rerun-if-changed={}", fens_path);

    // Read the file
    let file = File::open(fens_path).expect("Failed to open fens.txt");
    let reader = BufReader::new(file);

    // Collect filtered lines
    let filtered: Vec<String> = reader
        .lines()
        .filter_map(|line| {
            let line = line.ok()?;

            if let Ok(fen) = Fen::from_str(&line) {
                if fen.into_position::<Chess>(CastlingMode::Standard).is_ok() {
                    return Some(line);
                }
            }
            None
        })
        .collect();

    // Write filtered lines back to the file (overwriting it)
    fs::write(fens_path, filtered.join("\n") + "\n").expect("Failed to write filtered fens.txt");
}
