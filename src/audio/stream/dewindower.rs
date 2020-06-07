use super::super::common::*;
use super::node::*;
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Getters, Serialize, Deserialize, Debug)]
pub struct Dewindower {
    inputs: Vec<InputPort>,
    outputs: Vec<OutputPort>,
    id: NodeId,
    #[serde(skip)]
    buffer: Vec<VecDeque<f32>>,
    out_chunk_size: usize,
}

impl Dewindower {
    pub fn new(out_chunk_size: usize) -> Self {
        Self {
            inputs: vec![],
            outputs: vec![],
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

    pub fn process_chunk(&mut self, chunk: SampleChunk) -> Vec<SampleChunk> {
        let chunk = match chunk {
            SampleChunk::Real(chunk) => chunk,
            _ => panic!("incompatible input"),
        };
        let delay = *chunk.window_info().clone().unwrap().delay();
        for c in 0..*chunk.metadata().channels() {
            if self.buffer.len() <= c {
                self.buffer.push(chunk.samples(c).to_vec().into());
            } else {
                for _ in 0..delay {
                    self.buffer[c].push_back(0.0);
                }
                for i in 0..*chunk.duration_samples() {
                    let l = self.buffer[c].len();
                    self.buffer[c][l - chunk.duration_samples() + i] +=
                        chunk.samples(c)[i] * delay as f32 / *chunk.duration_samples() as f32;
                }
            }
        }
        let mut dewindowed_chunks = vec![];
        while self.buffer[0].len() >= self.out_chunk_size * 2 {
            let mut dewindowed_chunk = GenericSampleChunk::from_flat_samples(
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
            .map(|c| SampleChunk::Real(c.clone()))
            .collect::<Vec<_>>()
    }
}

impl NodeTrait for Dewindower {
    fn id(&self) -> NodeId {
        self.id
    }
    fn inputs(&self) -> &[InputPort] {
        &self.inputs
    }
    fn outputs(&self) -> &[OutputPort] {
        &self.outputs
    }
    fn inputs_mut(&mut self) -> &mut [InputPort] {
        &mut self.inputs
    }
    fn outputs_mut(&mut self) -> &mut [OutputPort] {
        &mut self.outputs
    }
    fn add_input(&mut self) -> Result<&mut InputPort, Box<dyn std::error::Error>> {
        if self.inputs.len() == 0 {
            self.inputs.push(InputPort::new(self.id));
            Ok(&mut self.inputs[0])
        } else {
            Err(Box::new(PortAdditionError))
        }
    }
    fn add_output(&mut self) -> Result<&mut OutputPort, Box<dyn std::error::Error>> {
        self.outputs.push(OutputPort::new(self.id));
        let l = self.outputs.len();
        Ok(&mut self.outputs[l - 1])
    }
    fn run_once(&mut self) {
        if self.inputs.len() != 1 {
            return;
        }
        while let Some(chunk) = self.inputs[0].try_recv().ok() {
            let chunks = self.process_chunk(chunk);
            for output in self.outputs().iter() {
                for chunk in chunks.iter() {
                    let _ = output.try_send(chunk.clone());
                }
            }
        }
    }
}
