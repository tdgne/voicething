use super::super::common::*;
use super::node::*;
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Getters, Serialize, Deserialize, Debug)]
pub struct Windower {
    io: NodeIo,
    id: NodeId,
    #[serde(skip)]
    buffer: Vec<VecDeque<f32>>,
    window_function: WindowFunction,
    window_size: usize,
    delay: usize,
}

impl HasNodeIo for Windower {
    fn node_io(&self) -> &NodeIo {
        &self.io
    }
    fn node_io_mut(&mut self) -> &mut NodeIo {
        &mut self.io
    }
}

impl Windower {
    pub fn new(window_function: WindowFunction, window_size: usize, delay: usize) -> Self {
        Self {
            io: NodeIo::new(),
            id: NodeId::new(),
            window_function,
            window_size,
            delay,
            buffer: vec![],
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
                if chunk.window_info().is_some() {
                    eprintln!("already windowed {}: {}", file!(), line!());
                    return vec![];
                }
                chunk
            }
            _ => {
                eprintln!("incompatible input {}: {}", file!(), line!());
                return vec![];
            }
        };
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
            let mut windowed_chunk = GenericDataChunk::from_flat_sata(
                &vec![0.0; self.buffer.len() * self.window_size],
                chunk.metadata().clone(),
            )
            .unwrap();
            windowed_chunk.set_window_info(Some(WindowInfo::new(
                self.window_function.clone(),
                self.delay,
            )));
            for (c, b) in self.buffer.iter().enumerate() {
                let samples = windowed_chunk.samples_mut(c);
                for (i, s) in b.iter().take(self.window_size).enumerate() {
                    samples[i] = *s
                        * match self.window_function {
                            WindowFunction::Rectangular => 1.0,
                            WindowFunction::Hanning => Self::hanning_window(i, self.window_size),
                            WindowFunction::Triangular => {
                                Self::triangular_window(i, self.window_size)
                            }
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
            .iter()
            .map(|c| DataChunk::Real(c.clone()))
            .collect::<Vec<_>>()
    }
}

impl NodeTrait for Windower {
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

#[cfg(test)]
mod test {
    use crate::audio::*;
    #[test]
    fn window_dewindow() {
        let c = DataChunk::Real(
            GenericDataChunk::from_flat_sata(&vec![1.0; 1024 * 4], AudioMetadata::new(2, 44100))
                .unwrap(),
        );
        let mut w = Windower::new(WindowFunction::Hanning, 300, 128);
        let mut dw = Dewindower::new(821);
        for n in w.process_chunk(c).into_iter() {
            let nc = match n.clone() {
                DataChunk::Real(n) => n,
                _ => panic!(),
            };
            assert_eq!(*nc.duration(), 300);
            assert_ne!(nc.samples(0)[1], 0.0);
            for n in dw.process_chunk(n).iter() {
                let nc = match n {
                    DataChunk::Real(n) => n,
                    _ => panic!(),
                };
                assert_eq!(*nc.duration(), 821);
                assert_ne!(nc.samples(0)[1], 0.0);
            }
        }
    }
}
