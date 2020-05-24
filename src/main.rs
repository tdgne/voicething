mod audio;
mod common;
mod config;
mod gui;
mod stream;

fn main() {
    let options = config::CommandLineOptions::parse_pub();
    let default_input = audio::default_input_device_name().map(|name| config::Input::Device(name));
    let default_output = audio::default_output_device_name().map(|name| config::Output::Device(name));
    let config = config::Config::new(options, default_input, default_output);

    let input = audio::spawn_input_thread(config.clone());
    let (gui_sender, gui_receiver) = stream::event_channel::<f32>();
    let playback_sink = audio::spawn_output_thread(gui_receiver);
    gui::main_loop(input, gui_sender, playback_sink, config);
}
