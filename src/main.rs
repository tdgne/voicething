mod audio;
mod common;
mod config;
mod gui;
mod stream;

fn main() {
    let options = config::CommandLineOptions::parse_pub();
    let input = if let Some(filename) = options.input_file() {
        Some(audio::Input::File(filename.clone()))
    } else {
        Some(audio::Input::Default)
    };
    let config = audio::AudioConfig::new(
        options,
        input,
        Some(audio::Output::Default),
        1024,
    );
    let (output_tx, input_rx, state) = audio::configure_audio_thread(None, config.clone());
    gui::main_loop(input_rx, output_tx, state);
}
