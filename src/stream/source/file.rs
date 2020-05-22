use crate::common::*;
use crate::stream::node::{Event, EventReceiver, EventSender};
use crate::stream::{Runnable, SingleOutputNode};
use getset::Getters;
use rodio;
use rodio::source::Source;
use std::error::Error;
use std::io::{Read, Seek};
use std::marker::PhantomData;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

#[derive(Getters)]
pub struct StaticSource<R: Read + Seek + Send> {
    #[getset(get = "pub")]
    samples: Vec<f32>,
    #[getset(get = "pub")]
    metadata: AudioMetadata,
    #[getset(get = "pub")]
    position: usize,
    #[getset(get = "pub", set = "pub")]
    chunk_duration: usize,
    #[getset(get = "pub", set = "pub")]
    sleep: bool,
    sender: Option<EventSender<f32>>,
    phantom: PhantomData<R>,
}

impl<R: Read + Seek + Send + 'static> StaticSource<R> {
    pub fn new(input: R, chunk_duration: usize) -> Result<Self, Box<dyn Error>> {
        let decoder = rodio::Decoder::new(input)?;
        let channels = decoder.channels() as usize;
        let sample_rate = decoder.sample_rate() as usize;
        let (sink, queue_out) = rodio::Sink::new_idle();
        sink.append(decoder);
        let samples = queue_out
            .take_while(move |_| !sink.empty())
            .collect::<Vec<_>>();
        let metadata = AudioMetadata::new(channels, sample_rate);
        Ok(Self {
            samples,
            metadata,
            position: 0,
            chunk_duration,
            phantom: PhantomData,
            sender: None,
            sleep: false,
        })
    }
}

impl<R: Read + Seek + Send + 'static> Runnable for StaticSource<R> {
    fn run(&mut self) {
        while let Some(_) = self.next() {
            if self.sleep {
                let seconds =
                    (self.chunk_duration as f64) / (*self.metadata().sample_rate() as f64);
                thread::sleep(Duration::from_micros((seconds * 1e6f64) as u64));
            }
        }
        if let Some(ref sender) = self.sender {
            sender.send(Event::Stop).unwrap();
        }
    }
}

impl<R: Read + Seek + Send> Iterator for StaticSource<R> {
    type Item = SampleChunk<f32>;

    fn next(&mut self) -> Option<SampleChunk<f32>> {
        let current_position = self.position;
        let next_position = current_position + self.chunk_duration * self.metadata.channels();
        if next_position > self.samples.len() {
            return None;
        }
        self.position = next_position;
        let chunk = SampleChunk::from_flat_samples(
            &self.samples[current_position..next_position],
            self.metadata.clone(),
        )
        .unwrap();

        if let Some(ref sender) = self.sender {
            sender.send(Event::Chunk(chunk.clone())).unwrap();
        }

        Some(chunk)
    }
}

impl<R: Read + Seek + Send + 'static> SingleOutputNode<f32> for StaticSource<R> {
    fn output(&mut self) -> EventReceiver<f32> {
        let (sender, receiver) = channel();
        self.sender = Some(sender);
        receiver
    }
}
