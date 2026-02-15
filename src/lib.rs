pub mod bot;
pub mod eval;
pub mod search;

/// Instructions for the search thread
pub enum SearchControl {
    // Search to a given depth
    ToDepth(u8),
    // Search for a approximate duration (in milliseconds)
    TimeLimit(u64),
}

/// Instructions for the search thread
pub enum SearchCommand {
    Start {
        position: shakmaty::Chess,
        control: SearchControl,
    },
    Stop,
    Quit,
    Reset,
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
