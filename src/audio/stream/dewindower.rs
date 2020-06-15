use super::super::common::*;
use super::node::*;
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Getters, Serialize, Deserialize, Debug)]
pub struct Dewindower {
    io: NodeIo,
    id: NodeId,
    #[serde(skip)]
    buffer: Vec<VecDeque<f32>>,
    out_chunk_size: usize,
}

impl HasNodeIo for Dewindower {
    fn node_io(&self) -> &NodeIo {
        &self.io
    }
    fn node_io_mut(&mut self) -> &mut NodeIo {
        &mut self.io
    }
}

impl Dewindower {
    pub fn new(out_chunk_size: usize) -> Self {
        Self {
            io: NodeIo::new(),
            id: NodeId::new(),
            buffer: vec![],
            out_chunk_size,
        }
    }

    fn id(&self) -> NodeId {
        self.id
    }

    fn triangular_window(x: usize, length: usize) -> f32 {
        let x = x as f32 / length as f32;
        1.0 - (x - 0.5).abs() * 2.0
    }

    fn hanning_window(x: usize, length: usize) -> f32 {
        let x = x as f32 / length as f32;
        0.5 - 0.5 * (2.0 * 3.141592 * x).cos()
    }

    pub fn process_chunk(&mut self, chunk: DataChunk) -> Vec<DataChunk> {
        let chunk = match chunk {
            DataChunk::Real(chunk) => {
                if chunk.window_info().is_none() {
                    eprintln!("not windowed {}: {}", file!(), line!());
                    return vec![];
                }
                chunk
            }
            _ => {
                eprintln!("incompatible input {}: {}", file!(), line!());
                return vec![];
            }
        };
        let delay = *chunk.window_info().clone().unwrap().delay();
        for c in 0..*chunk.metadata().channels() {
            if self.buffer.len() <= c {
                self.buffer.push(chunk.samples(c).to_vec().into());
            } else {
                for _ in 0..delay {
                    self.buffer[c].push_back(0.0);
                }
                for i in 0..*chunk.duration() {
                    let l = self.buffer[c].len();
                    self.buffer[c][l - chunk.duration() + i] +=
                        chunk.samples(c)[i] * delay as f32 / *chunk.duration() as f32;
                }
            }
        }
        let mut dewindowed_chunks = vec![];
        while self.buffer[0].len() >= self.out_chunk_size * 2 {
            let mut dewindowed_chunk = GenericDataChunk::from_flat_sata(
                &vec![0.0; self.buffer.len() * self.out_chunk_size],
                chunk.metadata().clone(),
            )
            .unwrap();
            for (c, b) in self.buffer.iter().enumerate() {
                let samples = dewindowed_chunk.samples_mut(c);
                for (i, s) in b.iter().take(self.out_chunk_size).enumerate() {
                    samples[i] = *s;
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
            .iter()
            .map(|c| DataChunk::Real(c.clone()))
            .collect::<Vec<_>>()
    }
}

impl NodeTrait for Dewindower {
    fn id(&self) -> NodeId {
        self.id
    }
    fn run_once(&mut self) {
        if self.inputs().len() != 1 {
            return;
        }
        while let Some(chunk) = self.inputs()[0].try_recv().ok() {
            let chunks = self.process_chunk(chunk);
            for output in self.outputs().iter() {
                for chunk in chunks.iter() {
                    let _ = output.try_send(chunk.clone());
                }
            }
        }
    }
}
