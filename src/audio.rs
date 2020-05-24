use cpal;
use cpal::traits::*;
use rodio;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::stream::{
    event_channel, EventReceiver, PlaybackSink, RecordingSource, Runnable, StaticSource,
};
use crate::{config, config::Config};

use crate::common::AudioMetadata;

pub fn default_input_device_name() -> Option<String> {
    let host = cpal::default_host();
    host.default_input_device().map(|d| d.name().unwrap())
}

pub fn input_device_names() -> Vec<String> {
    let host = cpal::default_host();
    host.devices()
        .unwrap()
        .flat_map(|device| {
            if device
                .supported_input_formats()
                .map_or(None, |mut formats| formats.next())
                .is_some()
            {
                device.name().ok()
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

pub fn default_output_device_name() -> Option<String> {
    let host = cpal::default_host();
    host.default_output_device().map(|d| d.name().unwrap())
}

pub fn output_device_names() -> Vec<String> {
    let host = cpal::default_host();
    host.devices()
        .unwrap()
        .flat_map(|device| {
            if device
                .supported_output_formats()
                .map_or(None, |mut formats| formats.next())
                .is_some()
            {
                device.name().ok()
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

pub fn spawn_output_thread(output: EventReceiver<f32>) -> Arc<Mutex<PlaybackSink>> {
    let playback_sink = Arc::new(Mutex::new(PlaybackSink::new(output)));
    {
        let playback_sink = playback_sink.clone();
        thread::spawn(move || {
            // cpal-related initialization needs to be called on a different thread
            // see https://gitter.im/tomaka/glutin?at=5dc6f493add5717a88da3652
            let device = rodio::default_output_device().unwrap();
            let rsink = rodio::Sink::new(&device);
            {
                let mut playback_sink = playback_sink.lock().unwrap();
                playback_sink.set_rodio_sink(rsink);
            }
            loop {
                {
                    let mut playback_sink = playback_sink.lock().unwrap();
                    playback_sink.run_once();
                }
                thread::sleep(std::time::Duration::from_millis(1));
            }
        });
    }
    playback_sink
}

pub fn spawn_input_thread(config: Config) -> EventReceiver<f32> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        tx.send(spawn_input_thread_internal(config)).unwrap();
    });
    rx.recv().unwrap()
}

fn spawn_input_thread_internal(config: Config) -> EventReceiver<f32> {
    match config.input() {
        Some(config::Input::File(input_file)) => {
            let file = std::fs::File::open(input_file).unwrap();
            let mut source = StaticSource::new(BufReader::new(file), 1024, true).unwrap();
            let output = source.output();
            thread::spawn(move || {
                source.run();
            });
            output
        }
        Some(config::Input::Device(name)) => {
            let mut source = RecordingSource::new(1024);
            let host = cpal::default_host();
            let event_loop = host.event_loop();
            let device = host
                .input_devices()
                .unwrap()
                .filter(|device| device.name().unwrap() == *name)
                .next()
                .unwrap();
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
        None => event_channel().1,
    }
}
