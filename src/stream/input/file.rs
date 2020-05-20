use crate::common::*;
use getset::Getters;
use rodio;
use rodio::source::Source;
use std::error::Error;
use std::io::{Read, Seek};
use std::marker::PhantomData;

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
        })
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
        Some(
            SampleChunk::from_flat_samples(
                &self.samples[current_position..next_position],
                self.metadata,
            )
            .unwrap(),
        )
    }
}
