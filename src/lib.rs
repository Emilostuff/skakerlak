use shakmaty_uci::UciInfoScore;

pub mod controller;
pub mod input;
pub mod search;

pub enum SearchCommand {
    Start { position: shakmaty::Chess },
    Stop,
    Quit,
}

pub struct Score {
    pub score: i32,
    pub mate: Option<i8>,
}

impl Score {
    pub fn new(score: i32, mate: Option<i8>) -> Self {
        Score { score, mate }
    }
}

impl Into<UciInfoScore> for Score {
    fn into(self) -> UciInfoScore {
        UciInfoScore {
            cp: Some(self.score),
            mate: self.mate,
            lower_bound: false,
            upper_bound: false,
        }
    }
}

pub enum SearchInfo {
    BestMove(shakmaty::Move),
    Info {
        depth: u8,
        pv: Vec<shakmaty::Move>,
        score: Score,
    },
}
