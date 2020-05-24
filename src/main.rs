mod audio;
mod common;
mod config;
mod gui;
mod stream;

fn main() {
    let options = config::Options::parse_pub();

    let input = audio::spawn_input_thread(options);
    let (gui_sender, gui_receiver) = stream::event_channel::<f32>();
    audio::spawn_output_thread(gui_receiver);
    gui::main_loop(input, gui_sender);
}
