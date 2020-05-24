mod audio;
mod common;
mod config;
mod gui;
mod stream;

use std::sync::{Arc, Mutex};

fn main() {
    let options = config::CommandLineOptions::parse_pub();
    let input = if let Some(filename) = options.input_file() {
        Some(config::Input::File(filename.clone()))
    } else {
        Some(config::Input::Default)
    };
    let config = Arc::new(Mutex::new(config::Config::new(
        options,
        input,
        Some(config::Output::Default),
    )));
    let (output_tx, input_rx) = audio::spawn_audio_thread(config.clone());
    gui::main_loop(input_rx, output_tx, config);
}
