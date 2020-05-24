use crate::stream::node::{Event, EventReceiver, Runnable};
use getset::Getters;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use crate::common::{AudioMetadata, SampleChunk};

#[derive(Getters)]
pub struct PlaybackSink {
    #[getset(get = "pub", set = "pub")]
    receiver: EventReceiver<f32>,
    buffer: Arc<Mutex<VecDeque<f32>>>,
}

impl PlaybackSink {
    pub fn new(receiver: EventReceiver<f32>) -> Self {
        Self {
            receiver,
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
        println!("{:?}", format);
        match output_buffer {
            cpal::UnknownTypeOutputBuffer::F32(buffer) => {
                self.send_buffer_f32(format, buffer);
            },
            _ => {}
        }
    }

    pub fn run_once(&mut self) -> bool {
        if let Ok(event) = self.receiver.recv() {
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
        false
    }
}
