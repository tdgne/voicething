use super::super::common::*;
use super::node::*;
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

#[derive(Getters, Serialize, Deserialize, Debug)]
pub struct Windower<S: Sample> {
    #[serde(skip)]
    input: Option<ChunkReceiver<S>>,
    #[serde(skip)]
    outputs: Vec<SyncChunkSender<S>>,
    #[serde(skip)]
    id: Uuid,
    #[serde(skip)]
    buffer: Vec<VecDeque<S>>,
    window_function: WindowFunction,
    window_size: usize,
    delay: usize,
}

impl<S: Sample> HasId for Windower<S> {
    fn id(&self) -> Uuid {
        self.id
    }
}

impl<S: Sample> Windower<S> {
    pub fn new(window_function: WindowFunction, window_size: usize, delay: usize) -> Self {
        Self {
            input: None,
            outputs: vec![],
            id: Uuid::new_v4(),
            window_function,
            window_size,
            delay,
            buffer: vec![],
        }
    }

    fn triangular_window(x: usize, length: usize) -> f32 {
        let x = x as f32 / length as f32;
        1.0 - (x - 0.5).abs() * 2.0
    }

    fn hanning_window(x: usize, length: usize) -> f32 {
        let x = x as f32 / length as f32;
        0.5 - 0.5 * (2.0 * 3.141592 * x).cos()
    }

    fn process_chunk_mul(&mut self, chunk: SampleChunk<S>) -> Vec<SampleChunk<S>> {
        for c in 0..*chunk.metadata().channels() {
            if self.buffer.len() <= c {
                self.buffer.push(vec![].into());
            }
            for s in chunk.samples(c).iter() {
                self.buffer[c].push_back(*s);
            }
        }
        let mut windowed_chunks = vec![];
        while self.buffer[0].len() >= self.window_size {
            let mut windowed_chunk: SampleChunk<S> = SampleChunk::from_flat_samples(
                &vec![S::from_f32(0.0).unwrap(); self.buffer.len() * self.window_size],
                chunk.metadata().clone(),
            )
            .unwrap();
            windowed_chunk.set_window_info(Some(WindowInfo::new(self.window_function.clone(), self.delay)));
            for (c, b) in self.buffer.iter().enumerate() {
                let samples = windowed_chunk.samples_mut(c);
                for (i, s) in b.iter().take(self.window_size).enumerate() {
                    samples[i] = *s * match self.window_function {
                        WindowFunction::Rectangular => S::from_f32(1.0f32).unwrap(),
                        WindowFunction::Hanning => S::from_f32(Self::hanning_window(i, self.window_size)).unwrap(),
                        WindowFunction::Triangular => S::from_f32(Self::triangular_window(i, self.window_size)).unwrap(),
                    };
                }
            }
            for b in self.buffer.iter_mut() {
                for _ in 0..self.delay {
                    b.pop_front();
                }
            }
            windowed_chunks.push(windowed_chunk);
        }
        windowed_chunks
    }
}

impl<S: Sample> SingleInput<S, S> for Windower<S> {
    fn input(&self) -> Option<&ChunkReceiver<S>> {
        self.input.as_ref()
    }

    fn outputs(&self) -> &[SyncChunkSender<S>] {
        self.outputs.as_ref()
    }

    fn set_input(&mut self, rx: Option<ChunkReceiver<S>>) {
        self.input = rx;
    }

    fn add_output(&mut self, tx: SyncChunkSender<S>) {
        self.outputs.push(tx);
    }

    fn process_chunk(&mut self, chunk: SampleChunk<S>) -> SampleChunk<S> {
        panic!("this should never be used")
    }

    fn run_once(&mut self) {
        if let Some(input) = self.input() {
            if let Some(chunk) = input.try_recv().ok() {
                let chunks = self.process_chunk_mul(chunk);
                for output in self.outputs().iter() {
                    for chunk in chunks.iter() {
                        let _ = output.try_send(chunk.clone());
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::audio::*;
    #[test]
    fn window_dewindow() {
        let c = SampleChunk::from_flat_samples(&vec![1.0; 1024*4], AudioMetadata::new(2, 44100)).unwrap();
        let mut w = Windower::new(WindowFunction::Hanning, 300, 128);
        let mut dw = Dewindower::new(821);
        for n in w.process_chunk(c).into_iter() {
            assert_eq!(*n.duration_samples(), 300);
            assert_ne!(n.samples(0)[1], 0.0);
            for n in dw.process_chunk(n).iter() {
                assert_eq!(*n.duration_samples(), 821);
                assert_ne!(n.samples(0)[1], 0.0);
            }
        }
    }
}
