use crate::{SearchCommand, SearchInfo};
use crossbeam_channel::{Receiver, Sender};
use shakmaty::{Chess, Move, Position, Role};

pub struct Searcher {
    cmd_rx: Receiver<SearchCommand>,
    info_tx: Sender<SearchInfo>,
}

struct MoveCandidate {
    mv: Move,
    next_position: Chess,
}

impl Searcher {
    pub fn new(cmd_rx: Receiver<SearchCommand>, info_tx: Sender<SearchInfo>) -> Self {
        Searcher { cmd_rx, info_tx }
    }

    pub fn run(self) {
        loop {
            match self.cmd_rx.recv().unwrap() {
                SearchCommand::Start { position, depth } => self.search(position, depth),
                SearchCommand::Stop => (),
                SearchCommand::Quit => break,
            }
        }
    }

    fn search(&self, position: Chess, depth: u8) {
        let mut moves: Vec<_> = position
            .legal_moves()
            .iter()
            .map(|m| {
                let mut pos = position.clone();
                pos.play_unchecked(m);
                (
                    MoveCandidate {
                        mv: m.clone(),
                        next_position: pos,
                    },
                    0,
                )
            })
            .collect();

        let mut pv = Vec::new();

        'outer: for d in 0..=depth - 1 {
            let mut nodes = 0;
            let mut best_pv = Vec::new();
            let mut best_score = i32::MIN + 1;

            for (move_candidate, score) in &mut moves {
                match self.cmd_rx.try_recv() {
                    Ok(SearchCommand::Start { .. }) | Ok(SearchCommand::Stop) => break 'outer,
                    Ok(SearchCommand::Quit) => return,
                    _ => (),
                };

                let (opponents_score, new_pv) = negamax(
                    &move_candidate.next_position,
                    d,
                    -i32::MAX,
                    -best_score,
                    0,
                    &mut nodes,
                );

                *score = -opponents_score;

                if *score > best_score {
                    best_score = *score;
                    best_pv = vec![move_candidate.mv.clone()];
                    best_pv.extend_from_slice(&new_pv);
                }
            }

            moves.sort_by_key(|(_, score)| -*score);
            pv = best_pv;

            self.info_tx
                .send(SearchInfo::Info {
                    depth: d + 1,
                    pv: pv.clone(),
                    score: best_score,
                    nodes,
                })
                .unwrap();
        }

        self.info_tx
            .send(SearchInfo::BestMove(pv[0].clone()))
            .unwrap();
    }
}

fn eval(pos: &Chess, ply: u8) -> i32 {
    if pos.is_checkmate() {
        return i32::MIN + 1 + ply as i32;
    } else if pos.has_insufficient_material(pos.turn()) || pos.is_stalemate() {
        return 0;
    }

    // otherwise just count material difference
    let mut material_diff = 0;

    for (_, piece) in pos.board().iter() {
        let value = match piece.role {
            Role::Pawn => 100,
            Role::Knight => 320,
            Role::Bishop => 330,
            Role::Rook => 500,
            Role::Queen => 900,
            Role::King => 0,
        };
        material_diff += value * if piece.color == pos.turn() { 1 } else { -1 };
    }

    material_diff
}

fn order(mut moves: Vec<Move>) -> Vec<Move> {
    moves.sort_by_key(|mv| -match mv {
        Move::Normal {
            capture: Some(_),
            promotion: Some(_),
            ..
        } => 10,
        Move::Normal {
            capture: None,
            promotion: Some(_),
            ..
        } => 7,
        Move::EnPassant { .. } => 6,
        Move::Normal {
            capture: Some(_),
            promotion: None,
            ..
        } => 5,
        Move::Normal {
            capture: None,
            promotion: None,
            ..
        } => 2,
        Move::Castle { .. } => 2,
        _ => 0,
    });
    moves
}

pub fn negamax(
    pos: &Chess,
    depth: u8,
    mut alpha: i32,
    beta: i32,
    ply: u8,
    nodes: &mut u64,
) -> (i32, Vec<Move>) {
    *nodes += 1;

    if depth == 0 || pos.is_game_over() {
        return (eval(pos, ply), vec![]);
    }

    let mut best_score = i32::MIN + 1;
    let mut best_line = vec![];

    for mv in order(pos.legal_moves().into_iter().collect()) {
        let mut new_pos = pos.clone();
        new_pos.play_unchecked(&mv);

        let (score, child_pv) = negamax(&new_pos, depth - 1, -beta, -alpha, ply + 1, nodes);

        let score = -score;

        if score > best_score {
            best_score = score;
            best_line = vec![mv.clone()];
            best_line.extend(child_pv);
        }

        alpha = alpha.max(score);
        if alpha >= beta {
            break;
        }
    }

    (best_score, best_line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::{fen::Fen, CastlingMode, Chess, Move, Position};
    use std::str::FromStr;

    fn find_mate(pos: Chess, in_n_moves: u8) -> Vec<Move> {
        let ply = in_n_moves * 2 - 1;
        let mut nodes = 0;
        let (score, pv) = negamax(&pos, ply, i32::MIN + 1, i32::MAX, 0, &mut nodes);

        assert_eq!(score, i32::MAX - ply as i32);
        assert_eq!(pv.len(), ply as usize);

        let mut test_pos = pos.clone();
        for mv in &pv {
            test_pos = test_pos.play(mv).unwrap();
        }
        assert!(test_pos.is_checkmate());
        pv
    }

    fn parse_fen(fen_str: &str) -> Chess {
        Fen::from_str(fen_str)
            .unwrap()
            .into_position(CastlingMode::Standard)
            .unwrap()
    }

    // Source for test problems used in testing: https://wtharvey.com/m8n3.txt

    #[test]
    // Madame de Remusat vs Napoleon I, Paris, 1802
    // 1... Bc5+ 2. Kxc5 Qb6+ 3. Kd5 Qd6#
    fn test_mate_in_3_remusat_napoleon_1802() {
        let pos = parse_fen("r1b1kb1r/pppp1ppp/5q2/4n3/3KP3/2N3PN/PPP4P/R1BQ1B1R b kq - 0 1");
        find_mate(pos, 3);
    }

    #[test]
    // William Evans vs Alexander MacDonnell, London, 1826
    // 1. Bb5+ c6 2. Qe6+ Qe7 3. Qxe7#
    fn test_mate_in_3_evans_macdonnell_1826() {
        let pos = parse_fen("r3k2r/ppp2Npp/1b5n/4p2b/2B1P2q/BQP2P2/P5PP/RN5K w kq - 1 0");
        find_mate(pos, 3);
    }

    #[test]
    // William Evans vs Alexander MacDonnell, London, 1829
    // 1. Qxh8+ Kxh8 2. Bf6+ Kg8 3. Re8#
    fn test_mate_in_3_evans_macdonnell_1829() {
        let pos = parse_fen("r1b3kr/ppp1Bp1p/1b6/n2P4/2p3q1/2Q2N2/P4PPP/RN2R1K1 w - - 1 0");
        find_mate(pos, 3);
    }

    #[test]
    // H Popert vs John Cochrane, London, 1841
    // 1... Qxf2+ 2. Rxf2 Rxf2+ 3. Kh1 Ng3#
    fn test_mate_in_3_popert_cochrane_1841() {
        let pos = parse_fen("r2n1rk1/1ppb2pp/1p1p4/3Ppq1n/2B3P1/2P4P/PP1N1P1K/R2Q1RN1 b - - 0 1");
        find_mate(pos, 3);
    }

    #[test]
    // Daniel Harrwitz vs Bernhard Horwitz, London, 1846
    // 1. Rxh7+ Kxh7 2. Rh1+ Kg7 3. Qh6#
    fn test_mate_in_3_harrwitz_horwitz_1846() {
        let pos = parse_fen("3q1r1k/2p4p/1p1pBrp1/p2Pp3/2PnP3/5PP1/PP1Q2K1/5R1R w - - 1 0");
        find_mate(pos, 3);
    }

    #[test]
    // Bernhard Horwitz vs Howard Staunton, London, 1846
    // 1... g6+ 2. hxg6 hxg6+ 3. Kf6 Nd7#
    fn test_mate_in_3_horwitz_staunton_1846() {
        let pos = parse_fen("6k1/ppp2ppp/8/2n2K1P/2P2P1P/2Bpr3/PP4r1/4RR2 b - - 0 1");
        find_mate(pos, 3);
    }

    #[test]
    // J Schulten vs Bernhard Horwitz, London, 1846
    // 1... Qf1+ 2. Kxf1 Bd3+ 3. Ke1 Rf1#
    fn test_mate_in_3_schulten_horwitz_1846() {
        let pos = parse_fen("rn3rk1/p5pp/2p5/3Ppb2/2q5/1Q6/PPPB2PP/R3K1NR b - - 0 1");
        find_mate(pos, 3);
    }

    #[test]
    // NN vs Henry Bird, England, 1850
    // 1... Ne2+ 2. Kh1 Ng3+ 3. Kg1 Rxf1#
    fn test_mate_in_3_nn_bird_1850() {
        let pos = parse_fen("N1bk4/pp1p1Qpp/8/2b5/3n3q/8/PPP2RPP/RNB1rBK1 b - - 0 1");
        find_mate(pos, 3);
    }

    #[test]
    // Adolf Anderssen vs Ernst Falkbeer, Berlin, 1851
    // 1. Re3+ Kxh2 2. Bxf4+ Kh1 3. Rh3#
    fn test_mate_in_3_anderssen_falkbeer_1851() {
        let pos = parse_fen("8/2p3N1/6p1/5PB1/pp2Rn2/7k/P1p2K1P/3r4 w - - 1 0");
        find_mate(pos, 3);
    }

    #[test]
    // Adolf Anderssen vs Lionel Kieseritzky, London, 1851
    // 1. Nxg7+ Kd8 2. Qf6+ Nxf6 3. Be7#
    fn test_mate_in_3_anderssen_kieseritzky_1851() {
        let pos = parse_fen("r1b1k1nr/p2p1ppp/n2B4/1p1NPN1P/6P1/3P1Q2/P1P1K3/q5b1 w - - 1 0");
        find_mate(pos, 3);
    }

    #[test]
    // Adolf Anderssen vs G Lepge, Leipzig, 1855
    // 1. Rxa6+ Rxa6 2. Qd7+ Kb6 3. c5#
    fn test_mate_in_3_anderssen_lepge_1855() {
        let pos = parse_fen("1q2r3/k4p2/prQ2b1p/R7/1PP1B1p1/6P1/P5K1/8 w - - 1 0");
        find_mate(pos, 3);
    }

    #[test]
    // Louis Paulsen vs NN, 1857
    // 1. Qxf7+ Nxf7 2. Bxf7+ Kf8 3. Ng6#
    fn test_mate_in_3_paulsen_nn_1857() {
        let pos = parse_fen("r1bqr1k1/ppp2pp1/3p4/4n1NQ/2B1PN2/8/P4PPP/b4RK1 w - - 1 0");
        find_mate(pos, 3);
    }

    #[test]
    // Paul Morphy vs NN, New Orleans, 1858
    // 1. Rb5+ Ka4 2. Qc2+ Ka3 3. Qb3#
    fn test_mate_in_3_morphy_nn_1858() {
        let pos = parse_fen("3r4/pp5Q/B7/k7/3q4/2b5/P4PPP/1R4K1 w - - 1 0");
        find_mate(pos, 3);
    }

    #[test]
    // A Saalbach vs H Pollmaecher, Leipzig, 1861
    // 1. Qe8+ Kxe8 2. Nf6+ Kd8 3. Nf7#
    fn test_mate_in_3_saalbach_pollmaecher_1861() {
        let pos = parse_fen("rnbk1b1r/ppqpnQ1p/4p1p1/2p1N1B1/4N3/8/PPP2PPP/R3KB1R w - - 1 0");
        find_mate(pos, 3);
    }

    #[test]
    // Frederic Deacon vs Valentine Green, London, 1862
    // 1. Ng6+ Kxh7 2. Nxf8+ Kg8 3. Qh7#
    fn test_mate_in_3_deacon_green_1862() {
        let pos = parse_fen("3rnr1k/p1q1b1pB/1pb1p2p/2p1P3/2P2N2/PP4P1/1BQ4P/4RRK1 w - - 1 0");
        find_mate(pos, 3);
    }

    #[test]
    // Louis Paulsen vs C Bollen, Dusseldorf, 1863
    // 1. Rxa6+ bxa6 2. Qxa6+ Ra7 3. Qc8#
    fn test_mate_in_3_paulsen_bollen_1863() {
        let pos = parse_fen("k7/1p1rr1pp/pR1p1p2/Q1pq4/P7/8/2P3PP/1R4K1 w - - 1 0");
        find_mate(pos, 3);
    }

    #[test]
    // Joseph Blackburne vs Adolf Anderssen, Vienna, 1873
    // 1. Qc8+ Kd6 2. Rxf6+ Re6 3. Rxe6#
    fn test_mate_in_3_blackburne_anderssen_1873() {
        let pos = parse_fen("Q4R2/3kr3/1q3n1p/2p1p1p1/1p1bP1P1/1B1P3P/2PBK3/8 w - - 1 0");
        find_mate(pos, 3);
    }
}
