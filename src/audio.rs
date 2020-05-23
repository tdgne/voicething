use rodio;
use std::io::BufReader;
use std::thread;

use crate::config::Options;
use crate::stream::{
    EventReceiver, PlaybackSink, RecordingSource, Runnable, SingleOutputNode, StaticSource,
};

use crate::common::AudioMetadata;

pub fn spawn_output_thread(output: EventReceiver<f32>) {
    let device = rodio::default_output_device().unwrap();
    let rsink = rodio::Sink::new(&device);
    let mut playback_sink = PlaybackSink::new(output, rsink);

    thread::spawn(move || {
        playback_sink.run();
    });
}

pub fn spawn_input_thread(options: Options) -> EventReceiver<f32> {
    let mut src: Box<dyn SingleOutputNode<f32>> = if let Some(input_file) = options.input_file() {
        let file = std::fs::File::open(input_file).unwrap();
        Box::new(StaticSource::new(BufReader::new(file), 2048, true).unwrap())
    } else {
        Box::new(RecordingSource::new(2048))
    };

    let output = src.output();

    thread::spawn(move || {
        src.run();
    });

    output
}
