use std::thread;

mod stream;
use stream::{Mixer, Runnable, ReceiverVolumePair, SingleOutputNode};

mod common;

mod gui;

mod audio;
use audio::*;

fn main() {
    let opts = audio::Opts::parse_pub();

    let input = spawn_audio_input_thread(opts);
    let mut mixer: Mixer<f32> = Mixer::new(vec![], common::AudioMetadata::new(2, 44100), 2048);
    mixer.add_input(ReceiverVolumePair {
        receiver: input,
        volume: 1.0,
    });
    let mixer_output = mixer.output();
    thread::spawn(move || {
        mixer.run();
    });
    spawn_audio_output_thread(mixer_output);

    let dummy: Mixer<f32> = Mixer::new(vec![], common::AudioMetadata::new(2, 44100), 2048);
    gui::main_loop(dummy);
}
