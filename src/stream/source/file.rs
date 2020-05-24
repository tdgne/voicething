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
use std::time::{Duration, SystemTime};

#[derive(Getters)]
pub struct StaticSource<R: Read + Seek + Send> {
    #[getset(get = "pub")]
    samples: Vec<f32>,
    #[getset(get = "pub")]
    metadata: AudioMetadata,
    #[getset(get = "pub", set = "pub")]
    position: usize,
    #[getset(get = "pub", set = "pub")]
    chunk_duration: usize,
    #[getset(get = "pub", set = "pub")]
    sleep: bool,
    #[getset(get = "pub", set = "pub")]
    repeat: bool,
    sender: Option<EventSender<f32>>,
    phantom: PhantomData<R>,
}

impl<R: Read + Seek + Send + 'static> StaticSource<R> {
    pub fn new(input: R, chunk_duration: usize, repeat: bool) -> Result<Self, Box<dyn Error>> {
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
            sleep: true,
            repeat,
        })
    }

    pub fn run(&mut self) {
        let mut sleep_start: Option<SystemTime> = None;
        let mut planned_sleep_time = Duration::from_secs(0);
        loop {
            while let Some(_) = self.next() {
                // sleep if repeat is on because I don't want to
                // infinitely push samples.
                if self.sleep || self.repeat {
                    let duration = {
                        let float_secs =
                            (self.chunk_duration as f64) / (*self.metadata().sample_rate() as f64);
                        Duration::from_micros((float_secs * 1e6f64) as u64)
                    };
                    let excess_time = if let Some(sleep_start) = sleep_start {
                        sleep_start.elapsed().unwrap() - planned_sleep_time
                    } else {
                        Duration::from_secs(0)
                    };
                    planned_sleep_time = duration - excess_time;
                    sleep_start = Some(SystemTime::now());
                    thread::sleep(planned_sleep_time);
                }
            }
            if !self.repeat {
                break;
            } else {
                self.position = 0;
            }
        }
        if let Some(ref sender) = self.sender {
            sender.send(Event::Stop).unwrap();
        }
    }

    pub fn output(&mut self) -> EventReceiver<f32> {
        let (sender, receiver) = channel();
        self.sender = Some(sender);
        receiver
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
