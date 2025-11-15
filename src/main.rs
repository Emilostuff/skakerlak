use crossbeam_channel::unbounded;
use skakarlak::{controller::Controller, input::InputListener, search::Searcher};
use std::thread;

fn main() {
    // Initialize channels
    let (input_tx, input_rx) = unbounded();
    let (cmd_tx, cmd_rx) = unbounded();
    let (info_tx, info_rx) = unbounded();

    // Spawn input listener thread
    thread::spawn(move || InputListener::new(input_tx).run());

    // Spawn search thread
    thread::spawn(move || Searcher::new(cmd_rx, info_tx).run());

    // Run controller on main thread
    Controller::new(input_rx, cmd_tx, info_rx, "engine.log").run();
}
