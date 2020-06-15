use crate::audio::common::*;
use std::collections::VecDeque;

pub struct Rechunker {
    buffer: VecDeque<f32>,
    out_channels: usize,
    out_sample_rate: usize,
}

pub fn format_chunk_channel(chunk: DataChunk, out_channels: usize) -> DataChunk {
    match chunk {
        DataChunk::Real(chunk) => {
            DataChunk::Real(format_chunk_channel_generic(chunk, out_channels))
        }
        DataChunk::Complex(chunk) => {
            DataChunk::Complex(format_chunk_channel_generic(chunk, out_channels))
        }
    }
}

pub fn format_chunk_sample_rate(chunk: DataChunk, out_sample_rate: usize) -> DataChunk {
    match chunk {
        DataChunk::Real(chunk) => {
            DataChunk::Real(format_chunk_sample_rate_generic(chunk, out_sample_rate))
        }
        DataChunk::Complex(chunk) => {
            DataChunk::Complex(format_chunk_sample_rate_generic(chunk, out_sample_rate))
        }
    }
}

fn format_chunk_channel_generic<S: Sample>(
    chunk: GenericDataChunk<S>,
    out_channels: usize,
) -> GenericDataChunk<S> {
    let out_format = AudioMetadata::new(out_channels, *chunk.metadata().sample_rate());
    let mut out_chunk = GenericDataChunk::from_flat_sata(
        &vec![S::zero(); out_channels * chunk.duration()],
        out_format,
    )
    .unwrap();
    let chunk_channels = *chunk.metadata().channels();
    match out_channels {
        1 => {
            for c in 0..chunk_channels {
                for (i, out_chunk_samples) in out_chunk.samples_mut(c).iter_mut().enumerate() {
                    *out_chunk_samples +=
                        chunk.samples(0)[i] * S::from_f32(1.0 / chunk_channels as f32).unwrap();
                }
            }
        }
        2 => {
            if chunk_channels == 1 {
                for (i, sample) in chunk.samples(0).iter().enumerate() {
                    out_chunk.samples_mut(0)[i] = *sample * S::from_f32(1.0).unwrap();
                    out_chunk.samples_mut(1)[i] = *sample * S::from_f32(1.0).unwrap();
                }
            } else if chunk_channels == 2 {
                for c in 0..chunk_channels {
                    for (i, sample) in chunk.samples(c).iter().enumerate() {
                        out_chunk.samples_mut(c)[i] = *sample * S::from_f32(1.0).unwrap();
                    }
                }
            } else {
                unimplemented!()
            }
        }
        _ => unimplemented!(),
    }
    out_chunk
}

fn format_chunk_sample_rate_generic<S: Sample>(
    chunk: GenericDataChunk<S>,
    out_sample_rate: usize,
) -> GenericDataChunk<S> {
    if out_sample_rate == *chunk.metadata().sample_rate() {
        return chunk;
    }
    let out_format = AudioMetadata::new(*chunk.metadata().channels(), out_sample_rate);
    let out_duration = (*chunk.duration() as f32 / *chunk.metadata().sample_rate() as f32
        * out_sample_rate as f32)
        .floor() as usize;
    let mut out_chunk = GenericDataChunk::from_flat_sata(
        &vec![S::zero(); chunk.metadata().channels() * out_duration],
        out_format,
    )
    .unwrap();
    let channels = *chunk.metadata().channels();
    for c in 0..channels {
        let in_samples = chunk.samples(c);
        let out_samples = out_chunk.samples_mut(c);
        for i in 0..out_duration {
            out_samples[i] = in_samples[(i as f32 / out_sample_rate as f32
                * *chunk.metadata().sample_rate() as f32)
                .floor() as usize]
        }
    }
    out_chunk
}

impl Rechunker {
    pub fn feed_chunk(&mut self, chunk: DataChunk) {
        if chunk.window_info().is_some() {
            eprintln!("input is windowed {}: {}", file!(), line!());
            return;
        }
        let chunk = format_chunk_sample_rate(chunk, self.out_sample_rate);
        let chunk = format_chunk_channel(chunk, self.out_channels);
        match chunk {
            DataChunk::Real(chunk) => {
                self.buffer.append(&mut chunk.flattened_data().into());
            }
            _ => {
                eprintln!("incompatible input {}: {}", file!(), line!());
            }
        };
    }

    pub fn pull_samples(&mut self, n: usize) -> Option<Vec<f32>> {
        let mut v = vec![];
        if n > self.buffer.len() {
            return None;
        }
        for _ in 0..n {
            v.push(self.buffer.pop_front().unwrap());
        }
        Some(v)
    }

    pub fn pull_chunk(&mut self, duration: usize) -> Option<DataChunk> {
        self.pull_samples(duration * self.out_channels)
            .map(|samples| {
                DataChunk::Real(
                    GenericDataChunk::from_flat_sata(
                        &samples,
                        AudioMetadata::new(self.out_channels, self.out_sample_rate),
                    )
                    .unwrap(),
                )
            })
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn new(out_channels: usize, out_sample_rate: usize) -> Self {
        Self {
            buffer: vec![].into(),
            out_channels,
            out_sample_rate,
        }
    }
}
