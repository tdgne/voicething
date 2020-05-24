use cpal;
use cpal::traits::*;
use rodio;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::stream::{
    event_channel, EventReceiver, EventSender, PlaybackSink, RecordingSource, Runnable,
    StaticSource,
};
use crate::{config, config::Config};

use crate::common::AudioMetadata;

fn do_in_thread<T: Send + 'static, F>(f: F) -> T
where
    F: Fn() -> T + Send + 'static,
{
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        tx.send(f()).unwrap();
    });
    rx.recv().unwrap()
}

pub fn spawn_audio_thread(config: Arc<Mutex<Config>>) -> (EventSender<f32>, EventReceiver<f32>) {
    // cpal-related initialization needs to be called on a different thread
    // see https://gitter.im/tomaka/glutin?at=5dc6f493add5717a88da3652
    do_in_thread(move || spawn_audio_thread_internal(config.clone()))
}

fn spawn_audio_thread_internal(
    config: Arc<Mutex<Config>>,
) -> (EventSender<f32>, EventReceiver<f32>) {
    let input_device_name;
    let output_device_name;
    {
        let config = config.lock().unwrap();
        input_device_name = config.input().clone();
        output_device_name = config.output().clone();
    }

    let host = cpal::default_host();
    let event_loop = host.event_loop();

    let mut recording_source = RecordingSource::new(1024);
    let input_info = match &input_device_name {
        Some(config::Input::Device(name)) => {
            let device = host
                .input_devices()
                .unwrap()
                .filter(|device| device.name().unwrap() == *name)
                .next()
                .unwrap();
            let format = device.default_input_format().unwrap();
            let stream_id = event_loop.build_input_stream(&device, &format).unwrap();
            Some((format, stream_id))
        }
        Some(config::Input::Default) => {
            let device = host.default_input_device().unwrap();
            let format = device.default_input_format().unwrap();
            let stream_id = event_loop.build_input_stream(&device, &format).unwrap();
            event_loop.play_stream(stream_id.clone()).unwrap();
            Some((format, stream_id))
        }
        _ => None,
    };
    let recording_rx = recording_source.output();

    let (output_tx, output_rx) = event_channel();
    let playback_sink = Arc::new(Mutex::new(PlaybackSink::new(output_rx)));
    let output_info = match output_device_name {
        Some(config::Output::Device(name)) => {
            let device = host
                .output_devices()
                .unwrap()
                .filter(|device| device.name().unwrap() == *name)
                .next()
                .unwrap();
            let format = device.default_input_format().unwrap();
            let stream_id = event_loop.build_input_stream(&device, &format).unwrap();
            event_loop.play_stream(stream_id.clone()).unwrap();
            Some((format, stream_id))
        }
        Some(config::Output::Default) => {
            let device = host.default_output_device().unwrap();
            let format = device.default_output_format().unwrap();
            let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
            event_loop.play_stream(stream_id.clone()).unwrap();
            Some((format, stream_id))
        }
        _ => None,
    };

    let file_rx = if let Some(config::Input::File(name)) = &input_device_name {
        let file = std::fs::File::open(name).unwrap();
        let mut source = StaticSource::new(BufReader::new(file), 1024, true).unwrap();
        let output = source.output();
        thread::spawn(move || {
            source.run();
        });
        Some(output)
    } else {
        None
    };

    {
        let playback_sink = playback_sink.clone();
        thread::spawn(move || loop {
            playback_sink.lock().unwrap().run_once();
            thread::sleep(std::time::Duration::from_millis(1));
        });
    }

    {
        let input_info = input_info.clone();
        thread::spawn(move || {
            event_loop.run(move |stream_id, mut stream_data| {
                if let Some((format, input_stream_id)) = &input_info {
                    if stream_id == input_stream_id.clone() {
                        match stream_data {
                            Ok(cpal::StreamData::Input { buffer }) => {
                                recording_source.send_buffer(format.clone(), buffer);
                            }
                            Err(e) => eprintln!("{}", e),
                            _ => {}
                        }
                    }
                } else if let Some((format, output_stream_id)) = &output_info {
                    if stream_id == *output_stream_id {
                        match stream_data {
                            Ok(cpal::StreamData::Output { ref mut buffer }) => {
                                playback_sink
                                    .lock()
                                    .unwrap()
                                    .send_buffer(format.clone(), buffer);
                            }
                            Err(e) => eprintln!("{}", e),
                            _ => {}
                        }
                    }
                }
            });
        });
    }

    if let Some(_) = input_info {
        (output_tx, recording_rx)
    } else {
        (output_tx, file_rx.unwrap())
    }
}
