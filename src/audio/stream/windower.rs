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

    pub fn input(&self) -> Option<&ChunkReceiver<S>> {
        self.input.as_ref()
    }

    pub fn outputs(&self) -> &[SyncChunkSender<S>] {
        self.outputs.as_ref()
    }

    pub fn set_input(&mut self, rx: Option<ChunkReceiver<S>>) {
        self.input = rx;
    }

    pub fn add_output(&mut self, tx: SyncChunkSender<S>) {
        self.outputs.push(tx);
    }

    fn triangular_window(x: usize, length: usize) -> f32 {
        let x = x as f32 / length as f32;
        1.0 - (x - 0.5).abs() * 2.0
    }

    fn hanning_window(x: usize, length: usize) -> f32 {
        let x = x as f32 / length as f32;
        0.5 - 0.5 * (2.0 * 3.141592 * x).cos()
    }

    pub fn process_chunk(&mut self, chunk: SampleChunk<S>) -> Vec<SampleChunk<S>> {
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
                &vec![S::from_f32(0.0).unwrap(); self.buffer.len() * self.buffer[0].len()],
                chunk.metadata().clone(),
            )
            .unwrap();
            for (c, b) in self.buffer.iter().enumerate() {
                let mut samples = windowed_chunk.samples_mut(c);
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
        }
        windowed_chunks
    }

    pub fn run_once(&mut self) {
        if let Some(input) = self.input() {
            if let Some(chunk) = input.try_recv().ok() {
                let chunks = self.process_chunk(chunk);
                for output in self.outputs().iter() {
                    for chunk in chunks.iter() {
                        let _ = output.try_send(chunk.clone());
                    }
                }
            }
        }
    }
}
