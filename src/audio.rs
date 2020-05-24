use cpal;
use cpal::traits::*;
use rodio;
use std::io::BufReader;
use std::thread;

use crate::config::Options;
use crate::stream::{
    EventReceiver, PlaybackSink, RecordingSource, Runnable, StaticSource,
};

use crate::common::AudioMetadata;

pub fn spawn_output_thread(output: EventReceiver<f32>) {
    thread::spawn(move || {
        // cpal-related initialization needs to be called on a different thread
        // see https://gitter.im/tomaka/glutin?at=5dc6f493add5717a88da3652
        let device = rodio::default_output_device().unwrap();
        let rsink = rodio::Sink::new(&device);
        let mut playback_sink = PlaybackSink::new(output, rsink);
        playback_sink.run();
    });
}

pub fn spawn_input_thread(options: Options) -> EventReceiver<f32> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        tx.send(spawn_input_thread_internal(options)).unwrap();
    });
    rx.recv().unwrap()
}

fn spawn_input_thread_internal(options: Options) -> EventReceiver<f32> {
    if let Some(input_file) = options.input_file() {
        let file = std::fs::File::open(input_file).unwrap();
        let mut source = StaticSource::new(BufReader::new(file), 2048, true).unwrap();
        let output = source.output();
        thread::spawn(move || {
            source.run();
        });
        output
    } else {
        let mut source = RecordingSource::new(2048);
        let host = cpal::default_host();
        let event_loop = host.event_loop();
        let device = host.default_input_device().unwrap();
        let format = device.default_input_format().unwrap();
        let stream_id = event_loop.build_input_stream(&device, &format).unwrap();
        event_loop.play_stream(stream_id.clone()).unwrap();
        source.formats_mut().insert(stream_id, format);
        let output = source.output();
        thread::spawn(move || {
            source.run(event_loop);
        });
        output
    }
}
