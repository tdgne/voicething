mod audio;
mod common;
mod config;
mod gui;
mod stream;

fn main() {
    let options = config::CommandLineOptions::parse_pub();
    let input = if let Some(filename) = options.input_file() {
        Some(config::Input::File(filename.clone()))
    } else {
        Some(config::Input::Default)
    };
    let config = config::AudioConfig::new(
        options,
        input,
        Some(config::Output::Default),
        1024,
    );
    let (output_tx, input_rx, state) = audio::spawn_audio_thread(None, config.clone());
    gui::main_loop(input_rx, output_tx, state);
}
