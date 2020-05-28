mod audio;
mod common;
mod config;
mod gui;
mod stream;
mod rechunker;

fn main() {
    let options = config::CommandLineOptions::parse_pub();
    let input = if let Some(filename) = options.input_file() {
        Some(audio::Input::File(filename.clone()))
    } else {
        Some(audio::Input::Default)
    };
    let config = audio::AudioConfig::from_command_line_options(
        options,
        input,
        Some(audio::Output::Default),
        1024,
    );
    let buffer = 2;
    let (tx_in, rx_in) = stream::event_sync_channel(buffer);
    let (tx_out, rx_out) = stream::event_channel();
    let host = audio::Host::new();
    let default_input_device_name = host.default_input_device_name();
    host.use_input_stream_from_device_name(default_input_device_name);
    host.set_sender(Some(tx_in));
    let default_output_device_name = host.default_output_device_name();
    host.use_output_stream_from_device_name(default_output_device_name);
    host.set_receiver(Some(rx_out));
    host.run();
    gui::main_loop(rx_in, tx_out);
}
