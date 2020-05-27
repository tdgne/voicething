use crate::stream::node::{Event, EventReceiver, Runnable};
use getset::{Getters, Setters};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use crate::common::{AudioMetadata, SampleChunk};

#[derive(Getters, Setters)]
pub struct PlaybackSink {
    #[getset(get = "pub", set = "pub")]
    receiver: Option<EventReceiver<f32>>,
    buffer: Arc<Mutex<VecDeque<f32>>>,
}

impl PlaybackSink {
    pub fn new() -> Self {
        Self {
            receiver: None,
            buffer: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    fn send_buffer_f32(
        &mut self,
        format: cpal::Format,
        out_buffer: &mut cpal::OutputBuffer<f32>,
    ) {
        let mut buffer = self.buffer.lock().unwrap();
        for i in 0..out_buffer.len() {
            if let Some(sample) = buffer.pop_front() {
                out_buffer[i] = sample;
            }
        }
    }

    pub fn send_buffer(&mut self, format: cpal::Format, output_buffer: &mut cpal::UnknownTypeOutputBuffer) {
        match output_buffer {
            cpal::UnknownTypeOutputBuffer::F32(buffer) => {
                self.send_buffer_f32(format, buffer);
            },
            _ => {}
        }
    }

    pub fn run_once(&mut self) -> bool {
        if let Some(ref receiver) = self.receiver {
            if let Ok(event) = receiver.recv() {
                match event {
                    Event::Chunk(chunk) => {
                        self.buffer
                            .lock()
                            .unwrap()
                            .append(&mut chunk.flattened_samples().into_iter().collect());
                    }
                    Event::Stop => return true,
                }
            }
        }
        false
    }
}
