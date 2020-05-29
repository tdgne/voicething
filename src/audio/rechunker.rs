use crate::audio::common::*;
use std::collections::VecDeque;

pub struct Rechunker {
    buffer: VecDeque<f32>,
    out_channels: usize,
    out_sample_rate: usize,
}

pub fn format_chunk_channel<S: Sample>(chunk: SampleChunk<S>, out_channels: usize) -> SampleChunk<S> {
    let out_format = AudioMetadata::new(out_channels, *chunk.metadata().sample_rate());
    let mut out_chunk = SampleChunk::from_flat_samples(
        &vec![S::zero(); out_channels * chunk.duration_samples()],
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

pub fn format_chunk_sample_rate<S: Sample>(
    chunk: SampleChunk<S>,
    out_sample_rate: usize,
) -> SampleChunk<S> {
    if out_sample_rate == *chunk.metadata().sample_rate() {
        return chunk;
    }
    let out_format = AudioMetadata::new(*chunk.metadata().channels(), out_sample_rate);
    let out_duration = (*chunk.duration_samples() as f32 / *chunk.metadata().sample_rate() as f32
        * out_sample_rate as f32)
        .floor() as usize;
    let mut out_chunk = SampleChunk::from_flat_samples(
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
    pub fn feed_chunk(&mut self, chunk: SampleChunk<f32>) {
        let chunk = format_chunk_sample_rate(chunk, self.out_sample_rate);
        let chunk = format_chunk_channel(chunk, self.out_channels);
        self.buffer.append(&mut chunk.flattened_samples().into());
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

    pub fn pull_chunk(&mut self, duration: usize) -> Option<SampleChunk<f32>> {
        self.pull_samples(duration * self.out_channels)
            .map(|samples| {
                SampleChunk::from_flat_samples(
                    &samples,
                    AudioMetadata::new(self.out_channels, self.out_sample_rate),
                )
                .unwrap()
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
