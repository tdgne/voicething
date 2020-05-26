use cpal;
use cpal::traits::*;
use rodio;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::thread;
use getset::Getters;

use crate::stream::{
    event_channel, EventReceiver, EventSender, PlaybackSink, RecordingSource, Runnable,
    StaticSource,
};
use crate::{config, config::AudioConfig};

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

pub fn spawn_audio_thread(
    state: Option<AudioState>,
    config: AudioConfig,
) -> (EventSender<f32>, EventReceiver<f32>, AudioState) {
    // cpal-related initialization needs to be called on a different thread
    // see https://gitter.im/tomaka/glutin?at=5dc6f493add5717a88da3652
    do_in_thread(move || spawn_audio_thread_internal(state.clone(), config.clone()))
}

enum InputSource {
    Recording(RecordingSource),
    Static(StaticSource<BufReader<std::fs::File>>),
}

#[derive(Clone, Getters)]
#[getset(get = "pub")]
pub struct AudioState {
    input_stream_id: Option<cpal::StreamId>,
    output_stream_id: Option<cpal::StreamId>,
    output_sample_rate: Option<usize>,
    config: AudioConfig,
}

fn configure_audio_input_internal(
    state: Option<AudioState>,
    config: AudioConfig,
    host: &cpal::Host,
    event_loop: &cpal::EventLoop,
) -> (
    Option<(cpal::Format, cpal::StreamId)>,
    InputSource,
    AudioState,
) {
    let input_device_name = config.input().clone();
    let chunk_size = config.chunk_size().clone();

    let recording_source = RecordingSource::new(chunk_size);
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
            Some((format, stream_id))
        }
        _ => None,
    };

    let static_source = if let Some(config::Input::File(name)) = &input_device_name {
        let file = std::fs::File::open(name).unwrap();
        Some(StaticSource::new(BufReader::new(file), chunk_size, true).unwrap())
    } else {
        None
    };

    if let Some(state) = state.clone() {
        if let Some(stream_id) = state.input_stream_id {
            event_loop.pause_stream(stream_id.clone()).unwrap();
            event_loop.destroy_stream(stream_id.clone());
        }
        if let Some(stream_id) = state.output_stream_id {
            event_loop.pause_stream(stream_id.clone()).unwrap();
            event_loop.destroy_stream(stream_id.clone());
        }
    }

    if let Some(input_info) = input_info {
        event_loop.play_stream(input_info.1.clone()).unwrap();
        let state = AudioState {
            input_stream_id: Some(input_info.1.clone()),
            output_stream_id: state.clone().map(|s| s.output_stream_id).unwrap_or(None),
            output_sample_rate: state.map(|s| s.output_sample_rate).unwrap_or(None),
            config,
        };
        (
            Some(input_info),
            InputSource::Recording(recording_source),
            state,
        )
    } else if let Some(static_source) = static_source {
        let state = AudioState {
            input_stream_id: None,
            output_stream_id: state.clone().map(|s| s.output_stream_id).unwrap_or(None),
            output_sample_rate: state.map(|s| s.output_sample_rate).unwrap_or(None),
            config,
        };
        (input_info, InputSource::Static(static_source), state)
    } else {
        unimplemented!()
    }
}

fn spawn_audio_thread_internal(
    state: Option<AudioState>,
    config: AudioConfig,
) -> (EventSender<f32>, EventReceiver<f32>, AudioState) {
    let output_device_name = config.output().clone();

    let host = cpal::default_host();
    let event_loop = host.event_loop();

    let (input_info, input_source, state) =
        configure_audio_input_internal(state, config, &host, &event_loop);
    let input_source = Arc::new(Mutex::new(input_source));
    let input_rx = {
        let mut input_source = input_source.lock().unwrap();
        match &mut *input_source {
            InputSource::Recording(source) => source.output(),
            InputSource::Static(source) => source.output(),
        }
    };

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

    let output_sample_rate = if let Some((ref format, _)) = output_info {
        Some(format.sample_rate.0 as usize)
    } else {
        None
    };

    let state = AudioState {
        config: state.config,
        input_stream_id: state.input_stream_id,
        output_stream_id: state.output_stream_id,
        output_sample_rate
    };

    {
        let playback_sink = playback_sink.clone();
        thread::spawn(move || loop {
            playback_sink.lock().unwrap().run_once();
            thread::sleep(std::time::Duration::from_millis(1));
        });
    }

    {
        let input_source = input_source.clone();
        let input_info = input_info.clone();
        thread::spawn(move || {
            event_loop.run(move |stream_id, mut stream_data| {
                if let Some((format, input_stream_id)) = &input_info {
                    if stream_id == input_stream_id.clone() {
                        match stream_data {
                            Ok(cpal::StreamData::Input { buffer }) => {
                                {
                                    let mut input_source = input_source.lock().unwrap();
                                    match &mut *input_source {
                                        InputSource::Recording(ref mut source) => {
                                            source.send_buffer(format.clone(), buffer);
                                        }
                                        _ => {
                                            // do nothing
                                        }
                                    }
                                }
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
    {
        let input_source = input_source.clone();
        let is_static_source = {
            if let &InputSource::Static(_) = &*input_source.lock().unwrap() {
                true
            } else {
                false
            }
        };

        if is_static_source {
            thread::spawn(move || {
                let mut input_source = input_source.lock().unwrap();
                match &mut *input_source {
                    InputSource::Static(ref mut source) => {
                        source.run();
                    }
                    _ => {
                        // do nothing
                    }
                }
            });
        }
    }

    (output_tx, input_rx, state)
}
