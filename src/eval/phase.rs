use shakmaty::{ByRole, Chess, Position};

const STARTING_SUM: f32 = 24.0;
const CONSERVATIVE_FACTOR: f32 = 1.5;

/// Determines how far the game has progressed from opening to endgame
pub struct Phase(f32);

impl Phase {
    pub fn new(pos: &Chess) -> Phase {
        let sum: u8 = pos
            .board()
            .material()
            .iter()
            .map(
                |ByRole {
                     knight,
                     bishop,
                     rook,
                     queen,
                     ..
                 }| { knight + bishop + rook * 2 + queen * 4 },
            )
            .sum();

        // Compute ratio
        let ratio = sum as f32 / STARTING_SUM;

        // Add conservatism
        let phase = ratio.powf(CONSERVATIVE_FACTOR);

        Phase(phase)
    }

    /// Outputs close the game is to the opening
    /// 1.0 = 100% opening
    /// 0.0 = 100% endgame
    pub fn opening(&self) -> f32 {
        self.0
    }

    /// Outputs close the game is to the endgame
    /// 1.0 = 100% endgame
    /// 0.0 = 100% opening
    pub fn endgame(&self) -> f32 {
        1.0 - self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::{fen::Fen, CastlingMode, Chess};
    use std::str::FromStr;

    fn parse_fen(fen_str: &str) -> Chess {
        Fen::from_str(fen_str)
            .unwrap()
            .into_position(CastlingMode::Standard)
            .unwrap()
    }

    #[test]
    fn test_phase_opening() {
        let pos = Chess::default();
        let phase = Phase::new(&pos);
        assert_eq!(phase.opening(), 1.0);
        assert_eq!(phase.endgame(), 0.0);
    }

    #[test]
    fn test_phase_endgame() {
        // Only kings and pawns
        let pos = parse_fen("4k3/8/8/3p4/3P4/8/8/4K3 w - - 0 1");
        let phase = Phase::new(&pos);
        assert_eq!(phase.opening(), 0.0);
        assert_eq!(phase.endgame(), 1.0);
    }

    #[test]
    fn test_phase_middlegame() {
        // Remove both queens from opening position
        let pos = parse_fen("rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNB1KBNR w KQkq - 0 1");
        let phase = Phase::new(&pos);
        assert!(
            phase.opening() > 0.0 && phase.opening() < 1.0,
            "Opening must be strictly between 0.0 and 1.0"
        );
        assert!(
            phase.endgame() > 0.0 && phase.endgame() < 1.0,
            "Endgame must be strictly between 0.0 and 1.0"
        );
    }
}
