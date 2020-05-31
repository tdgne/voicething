use super::super::common::*;
use super::node::*;
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

#[derive(Getters, Serialize, Deserialize, Debug)]
pub struct Dewindower<S: Sample> {
    #[serde(skip)]
    input: Option<ChunkReceiver<S>>,
    #[serde(skip)]
    outputs: Vec<SyncChunkSender<S>>,
    #[serde(skip)]
    id: Uuid,
    #[serde(skip)]
    buffer: Vec<VecDeque<S>>,
    out_chunk_size: usize,
}

impl<S: Sample> HasId for Dewindower<S> {
    fn id(&self) -> Uuid {
        self.id
    }
}

impl<S: Sample> Dewindower<S> {
    pub fn new(out_chunk_size: usize) -> Self {
        Self {
            input: None,
            outputs: vec![],
            id: Uuid::new_v4(),
            buffer: vec![],
            out_chunk_size,
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
        let delay = *chunk.window_info().clone().unwrap().delay();
        for c in 0..*chunk.metadata().channels() {
            if self.buffer.len() <= c {
                self.buffer.push(vec![S::from_f32(0.0).unwrap(); chunk.duration_samples() - delay].into());
            }
            for _ in 0..delay {
                self.buffer[c].push_back(S::from_f32(0.0).unwrap());
            }
            let l = self.buffer[c].len();
            for (i, b) in chunk.samples(c).iter().enumerate() {
                self.buffer[c][l - chunk.duration_samples() + i] += *b;
            }
        }
        let mut dewindowed_chunks = vec![];
        while self.buffer[0].len() >= self.out_chunk_size + chunk.duration_samples() {
            let mut dewindowed_chunk: SampleChunk<S> = SampleChunk::from_flat_samples(
                &vec![S::from_f32(0.0).unwrap(); self.buffer.len() * self.out_chunk_size],
                chunk.metadata().clone(),
            )
            .unwrap();
            for (c, b) in self.buffer.iter().enumerate() {
                let samples = dewindowed_chunk.samples_mut(c);
                for (i, s) in b.iter().take(self.out_chunk_size).enumerate() {
                    samples[i] = *s * S::from_f32(delay as f32 / self.out_chunk_size as f32).unwrap();
                }
            }
            for b in self.buffer.iter_mut() {
                for _ in 0..self.out_chunk_size {
                    b.pop_front();
                }
            }
            dewindowed_chunks.push(dewindowed_chunk);
        }
        dewindowed_chunks
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
