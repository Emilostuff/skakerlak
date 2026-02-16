use colour::*;
use shakmaty::fen::Fen;
use shakmaty::{CastlingMode, Chess};
use std::fmt::Display;
use std::str::FromStr;

pub fn plot_quantity<T: Display>(name: &str, data: Vec<T>) {
    blue_ln_bold!("    {} ", name);
    black!("    Min: ");
    yellow!("{:.1}", data[0]);
    black!("   Median: ");
    white_bold!("{:.1}", data[data.len() / 2]);
    black!("   Max: ");
    green_ln!("{:.1}", data[data.len() - 1]);
    println!();
}

#[allow(dead_code)]
pub struct Measurements {
    pub depth: u8,
    pub time_ms: u128,
    pub nodes: u64,
}

pub fn parse_fen(fen_str: &str) -> Chess {
    Fen::from_str(fen_str)
        .unwrap()
        .into_position(CastlingMode::Standard)
        .unwrap()
}

pub fn pad(strings: &[&str]) -> usize {
    strings.iter().map(|x| x.len()).max().unwrap()
}
