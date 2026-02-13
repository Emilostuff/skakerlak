use crate::{SearchCommand, SearchControl, SearchInfo};
use chrono::Local;
use crossbeam_channel::{select, Receiver, Sender};
use shakmaty::{CastlingMode, Chess, Position};
use shakmaty_uci::{UciInfo, UciInfoScore, UciMessage, UciMove, UciSearchControl};
use std::{fs::OpenOptions, io::Write};

/// Handles incoming commands, sends outgoing messages and produces runtime logs.
pub struct Controller {
    input_rx: Receiver<UciMessage>,
    cmd_tx: Sender<SearchCommand>,
    info_rx: Receiver<SearchInfo>,
    position: Chess,
    log_file: &'static str,
}

impl Controller {
    pub fn new(
        input_rx: Receiver<UciMessage>,
        cmd_tx: Sender<SearchCommand>,
        info_rx: Receiver<SearchInfo>,
        log_file: &'static str,
    ) -> Self {
        let controller = Controller {
            input_rx,
            cmd_tx,
            info_rx,
            position: Chess::default(),
            log_file,
        };

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        controller.log("");
        controller.log(&format!("------ Engine started at {} ------", timestamp));

        controller
    }

    /// Runs the controller.
    pub fn run(mut self) {
        loop {
            select! {
                recv(self.input_rx) -> cmd => {
                    let cmd = cmd.unwrap();
                    self.log(&format!(" IN: '{}'", &cmd));
                    if self.handle_input(cmd) {
                        break;
                    }
                }

                recv(self.info_rx) -> info => self.handle_info(info.unwrap()),
            }
        }
    }

    fn send(&self, msg: UciMessage) {
        println!("{msg}");
        self.log(&format!("OUT: '{}'", msg));
    }

    fn send_init_response(&self) {
        self.send(UciMessage::Id {
            name: Some(format!("Skakarlak {}", env!("CARGO_PKG_VERSION"))),
            author: None,
        });
        self.send(UciMessage::Id {
            name: None,
            author: Some("Emil Skydsgaard".into()),
        });
        self.send(UciMessage::UciOk);
    }

    fn handle_input(&mut self, message: UciMessage) -> bool {
        match message {
            UciMessage::Quit => return true,
            UciMessage::Uci => self.send_init_response(),
            UciMessage::IsReady => self.send(UciMessage::ReadyOk),
            UciMessage::UciNewGame => {
                self.position = Chess::default();
            }
            UciMessage::Position { fen, moves, .. } => {
                let mut position = if let Some(fen) = fen {
                    fen.into_position(CastlingMode::Standard).unwrap()
                } else {
                    Chess::default()
                };

                for mv in moves {
                    let m = mv.to_move(&position).unwrap();
                    position = position.play(m).unwrap();
                }
                self.position = position;
            }
            UciMessage::Go {
                search_control:
                    Some(UciSearchControl {
                        depth: Some(depth), ..
                    }),
                ..
            } => self
                .cmd_tx
                .send(SearchCommand::Start {
                    position: self.position.clone(),
                    control: SearchControl::ToDepth(depth),
                })
                .unwrap(),
            UciMessage::Go { .. } => self
                .cmd_tx
                .send(SearchCommand::Start {
                    position: self.position.clone(),
                    control: SearchControl::TimeLimit(2000),
                })
                .unwrap(),
            UciMessage::Stop => self.cmd_tx.send(SearchCommand::Stop).unwrap(),

            _ => (), // Other commands are not handled here.
        }
        false
    }

    fn handle_info(&mut self, message: SearchInfo) {
        match message {
            SearchInfo::BestMove(mv) => self.send(UciMessage::BestMove {
                best_move: UciMove::from_move(mv, CastlingMode::Standard),
                ponder: None,
            }),
            SearchInfo::Info {
                depth,
                pv,
                score,
                nodes,
            } => {
                let info_msg = UciMessage::Info(UciInfo {
                    depth: Some(depth),
                    score: Some(UciInfoScore {
                        cp: Some(score),
                        ..Default::default()
                    }),
                    pv: pv
                        .into_iter()
                        .map(|mv| UciMove::from_move(mv, CastlingMode::Standard))
                        .collect(),
                    nodes: Some(nodes),

                    ..Default::default()
                });

                self.send(info_msg);
            }
        }
    }

    fn log(&self, line: &str) {
        let mut log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.log_file)
            .unwrap();
        writeln!(&mut log_file, "{}", line).unwrap()
    }
}

impl Drop for Controller {
    fn drop(&mut self) {
        self.log("------ Engine closed ------");
    }
}
