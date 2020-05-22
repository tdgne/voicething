use clap::Clap;
use rodio;
use std::io::BufReader;
use std::thread;

use crate::stream::{
    Mixer, PlaybackSink, ReceiverVolumePair, RecordingSource, SingleOutputNode, StaticSource,
    EventReceiver, Runnable
};

use crate::common::AudioMetadata;

#[derive(Clap, Clone)]
#[clap(version = "0.0", author = "tdgne")]
pub struct Opts {
    #[clap(short, long)]
    input_file: Option<String>,
    #[clap(short, long)]
    output_file: Option<String>,
}

impl Opts {
    pub fn parse_pub() -> Self {
        Self::parse()
    }
}

pub fn spawn_audio_output_thread(output: EventReceiver<f32>) {
    let device = rodio::default_output_device().unwrap();
    let rsink = rodio::Sink::new(&device);
    let mut output_thread_receiver: Mixer<f32> =
        Mixer::new(vec![], AudioMetadata::new(2, 44100), 2048);
    output_thread_receiver.add_input(ReceiverVolumePair {
        receiver: output,
        volume: 1.0,
    });
    let mut playback_sink = PlaybackSink::new(output_thread_receiver.output(), rsink);

    thread::spawn(move || {
        output_thread_receiver.run();
    });

    thread::spawn(move || {
        playback_sink.run();
    });
}

pub fn spawn_audio_input_thread(opts: Opts) -> EventReceiver<f32> {
    let mut src: Box<dyn SingleOutputNode<f32>> = if let Some(input_file) = opts.input_file {
        let file = std::fs::File::open(input_file).unwrap();
        Box::new(StaticSource::new(BufReader::new(file), 2048).unwrap())
    } else {
        Box::new(RecordingSource::new(2048))
    };

    let output = src.output();

    thread::spawn(move || {
        src.run();
    });

    output
}
