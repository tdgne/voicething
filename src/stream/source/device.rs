use crate::common::*;
use crate::stream::node::{Event, EventReceiver, EventSender};
use crate::stream::{Runnable, SingleOutputNode};
use cpal::traits::{EventLoopTrait, HostTrait};
use getset::Getters;
use std::collections::{HashMap, VecDeque};
use std::sync::mpsc::channel;

#[derive(Getters)]
pub struct RecordingSource {
    sender: Option<EventSender<f32>>,
    buffer: VecDeque<f32>,
    chunk_duration: usize,
    #[getset(get = "pub")]
    formats: HashMap<cpal::StreamId, cpal::Format>,
}

impl RecordingSource {
    pub fn new(chunk_duration: usize) -> Self {
        Self {
            sender: None,
            buffer: VecDeque::new(),
            chunk_duration,
            formats: HashMap::new(),
        }
    }

    pub fn formats_mut(&mut self) -> &mut HashMap<cpal::StreamId, cpal::Format> {
        &mut self.formats
    }

    pub fn send_buffer(&mut self, format: cpal::Format, buffer: &cpal::UnknownTypeInputBuffer) {
        let sender = self.sender.as_mut().unwrap();
        let metadata =
            AudioMetadata::new(format.channels as usize, format.sample_rate.0 as usize);
        match buffer {
            cpal::UnknownTypeInputBuffer::F32(input_buffer) => {
                for sample in input_buffer.iter() {
                    self.buffer.push_back(*sample);
                }
                let channels = metadata.channels();
                let chunk_duration = self.chunk_duration;
                if self.buffer.len() >= channels * chunk_duration {
                    let mut samples = vec![];
                    for _ in 0..channels * chunk_duration {
                        samples.push(self.buffer.pop_front().unwrap());
                    }

                    let chunk =
                        SampleChunk::from_flat_samples(&samples, metadata.clone()).unwrap();

                    sender.send(Event::Chunk(chunk)).unwrap();
                }
            }
            _ => unimplemented!(),
        }
    }

    pub fn output(&mut self) -> EventReceiver<f32> {
        let (sender, receiver) = channel();
        self.sender = Some(sender);
        receiver
    }
}
