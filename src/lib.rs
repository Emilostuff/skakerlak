pub mod bot;
pub mod eval;
pub mod search;

/// Instructions for the search thread
pub enum SearchCommand {
    Start {
        position: shakmaty::Chess,
        depth: u8,
    },
    Stop,
    Quit,
}

/// Search information to be logged
pub enum SearchInfo {
    BestMove(shakmaty::Move),
    Info {
        depth: u8,
        pv: Vec<shakmaty::Move>,
        score: i32,
        nodes: u64,
    },
}
