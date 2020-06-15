use cpal;
use cpal::traits::*;
use getset::{Getters, Setters};
use std::sync::{
    mpsc,
    mpsc::{Receiver, SyncSender},
    Arc, Mutex,
};
use std::thread;

use crate::audio::common::{AudioMetadata, DataChunk, GenericDataChunk};
use crate::audio::rechunker::*;

#[derive(Clone)]
pub struct StreamInfo {
    stream_id: cpal::StreamId,
    format: cpal::Format,
    device_name: String,
}

pub struct Host {
    host: Arc<cpal::Host>,
    event_loop: Arc<cpal::EventLoop>,
    input_stream: Arc<Mutex<Option<StreamInfo>>>,
    output_stream: Arc<Mutex<Option<StreamInfo>>>,
    sender: Arc<Mutex<Option<SyncSender<DataChunk>>>>,
    receiver: Arc<Mutex<Option<Receiver<DataChunk>>>>,
    rechunker: Arc<Mutex<Option<Rechunker>>>,
}

impl Host {
    pub fn new() -> Self {
        let host = Arc::new(cpal::default_host());
        let event_loop = Arc::new(host.event_loop());
        Self {
            host,
            event_loop,
            input_stream: Arc::new(Mutex::new(None)),
            output_stream: Arc::new(Mutex::new(None)),
            sender: Arc::new(Mutex::new(None)),
            receiver: Arc::new(Mutex::new(None)),
            rechunker: Arc::new(Mutex::new(None)),
        }
    }

    pub fn current_input_device_name(&self) -> Option<String> {
        self.input_stream
            .lock()
            .unwrap()
            .as_ref()
            .map(|s| s.device_name.clone())
    }

    pub fn current_output_device_name(&self) -> Option<String> {
        self.output_stream
            .lock()
            .unwrap()
            .as_ref()
            .map(|s| s.device_name.clone())
    }

    pub fn input_device_names(&self) -> Vec<String> {
        let host = self.host.clone();
        do_in_thread(move || {
            host.input_devices()
                .unwrap()
                .flat_map(|d| d.name().ok())
                .collect::<Vec<_>>()
        })
    }

    pub fn output_device_names(&self) -> Vec<String> {
        let host = self.host.clone();
        do_in_thread(move || {
            host.output_devices()
                .unwrap()
                .flat_map(|d| d.name().ok()) // FIXME: mac's multiple output devices will be discarded here
                .collect::<Vec<_>>()
        })
    }

    pub fn default_input_device_name(&self) -> Option<String> {
        let host = self.host.clone();
        do_in_thread(move || host.default_input_device().map(|d| d.name().unwrap()))
    }

    pub fn default_output_device_name(&self) -> Option<String> {
        let host = self.host.clone();
        do_in_thread(move || host.default_output_device().map(|d| d.name().unwrap()))
    }

    pub fn use_input_stream_from_device_name(&self, name: String) {
        let host = self.host.clone();
        let event_loop = self.event_loop.clone();
        let input_stream = self.input_stream.clone();
        let stream_info = do_in_thread(move || {
            if let Some(ref info) = &*input_stream.lock().unwrap() {
                let old_stream_id = info.stream_id.clone();
                event_loop.pause_stream(old_stream_id.clone()).unwrap();
                event_loop.destroy_stream(old_stream_id);
            }
            let device = host
                .input_devices()
                .unwrap()
                .find(|d| d.name().unwrap() == name)
                .unwrap();
            let device_name = device.name().unwrap();
            let format = device.default_input_format().unwrap();
            let stream_id = event_loop.build_input_stream(&device, &format).unwrap();
            event_loop.play_stream(stream_id.clone()).unwrap();
            StreamInfo {
                stream_id,
                format,
                device_name,
            }
        });
        *self.input_stream.lock().unwrap() = Some(stream_info);
    }

    pub fn use_output_stream_from_device_name(&self, name: String) {
        let host = self.host.clone();
        let event_loop = self.event_loop.clone();
        let output_stream = self.output_stream.clone();
        let stream_info = do_in_thread(move || {
            if let Some(ref info) = &*output_stream.lock().unwrap() {
                let old_stream_id = info.stream_id.clone();
                event_loop.pause_stream(old_stream_id.clone()).unwrap();
                event_loop.destroy_stream(old_stream_id);
            }
            let device = host
                .output_devices()
                .unwrap()
                .find(|d| d.name().unwrap() == name)
                .unwrap();
            let device_name = device.name().unwrap();
            let format = device.default_output_format().unwrap();
            let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
            event_loop.play_stream(stream_id.clone()).unwrap();
            StreamInfo {
                stream_id,
                format,
                device_name,
            }
        });
        *self.rechunker.lock().unwrap() = Some(Rechunker::new(
            stream_info.format.channels as usize,
            stream_info.format.sample_rate.0 as usize,
        ));
        *self.output_stream.lock().unwrap() = Some(stream_info);
    }

    pub fn set_sender(&self, sender: Option<SyncSender<DataChunk>>) {
        *self.sender.lock().unwrap() = sender;
    }

    pub fn set_receiver(&self, receiver: Option<Receiver<DataChunk>>) {
        *self.receiver.lock().unwrap() = receiver;
    }

    pub fn run(&self) {
        let event_loop = self.event_loop.clone();
        let input_stream = self.input_stream.clone();
        let output_stream = self.output_stream.clone();
        let sender = self.sender.clone();
        let receiver = self.receiver.clone();
        let rechunker = self.rechunker.clone();
        thread::spawn(move || {
            let input_stream = input_stream.clone();
            let output_stream = output_stream.clone();
            let sender = sender.clone();
            let receiver = receiver.clone();
            let rechunker = rechunker.clone();
            event_loop.run(move |stream_id, mut stream_data| {
                if let Some(StreamInfo {
                    stream_id: ref input_stream_id,
                    ref format,
                    device_name: _,
                }) = &*input_stream.lock().unwrap()
                {
                    if stream_id == *input_stream_id {
                        match &stream_data {
                            Ok(cpal::StreamData::Input { buffer }) => {
                                if let Some(ref sender) = &*sender.lock().unwrap() {
                                    let chunk = chunk_from_buffer(format.clone(), buffer);
                                    let _ = sender.try_send(chunk);
                                }
                            }
                            Err(e) => eprintln!("{}", e),
                            _ => {}
                        }
                    }
                }
                if let Some(StreamInfo {
                    stream_id: ref output_stream_id,
                    ref format,
                    device_name: _,
                }) = &*output_stream.lock().unwrap()
                {
                    if stream_id == *output_stream_id {
                        match &mut stream_data {
                            Ok(cpal::StreamData::Output { ref mut buffer }) => {
                                if let Some(ref receiver) = &*receiver.lock().unwrap() {
                                    while let Ok(chunk) = receiver.try_recv() {
                                        if let Some(ref mut rechunker) =
                                            &mut *rechunker.lock().unwrap()
                                        {
                                            rechunker.feed_chunk(chunk);
                                        }
                                    }
                                    if let Some(ref mut rechunker) = &mut *rechunker.lock().unwrap()
                                    {
                                        if let Some(chunk) = rechunker
                                            .pull_chunk(buffer.len() / format.channels as usize)
                                        {
                                            write_chunk_to_buffer(chunk, buffer);
                                        }
                                    }
                                }
                            }
                            Err(e) => eprintln!("{}", e),
                            _ => {}
                        }
                    }
                }
            });
        });
    }
}

fn chunk_from_buffer(format: cpal::Format, buffer: &cpal::UnknownTypeInputBuffer) -> DataChunk {
    let metadata = AudioMetadata::new(format.channels as usize, format.sample_rate.0 as usize);
    match buffer {
        cpal::UnknownTypeInputBuffer::U16(buffer) => unimplemented!(),
        cpal::UnknownTypeInputBuffer::I16(buffer) => unimplemented!(),
        cpal::UnknownTypeInputBuffer::F32(buffer) => {
            DataChunk::Real(GenericDataChunk::from_flat_sata(buffer, metadata).unwrap())
        }
    }
}

fn write_chunk_to_buffer(chunk: DataChunk, buffer: &mut cpal::UnknownTypeOutputBuffer) {
    match buffer {
        cpal::UnknownTypeOutputBuffer::U16(buffer) => unimplemented!(),
        cpal::UnknownTypeOutputBuffer::I16(buffer) => unimplemented!(),
        cpal::UnknownTypeOutputBuffer::F32(buffer) => {
            let samples = match chunk {
                DataChunk::Real(chunk) => chunk.flattened_data(),
                _ => panic!("Incompatible input"),
            };
            for (i, b) in buffer.iter_mut().enumerate() {
                *b = samples[i];
            }
        }
    }
}

#[derive(Clone)]
pub enum Input {
    Default,
    Device(String),
    File(String),
}

#[derive(Clone)]
pub enum Output {
    Default,
    Device(String),
}

// cpal-related initialization needs to be called on a different thread
// see https://gitter.im/tomaka/glutin?at=5dc6f493add5717a88da3652
fn do_in_thread<T: Send + 'static, F>(f: F) -> T
where
    F: Fn() -> T + Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        tx.send(f()).unwrap();
    });
    rx.recv().unwrap()
}
