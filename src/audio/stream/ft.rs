use super::super::common::*;
use super::node::*;
use getset::Getters;
use rustfft::num_complex::Complex32;
use rustfft::num_traits::*;
use rustfft::FFTplanner;
use serde::{Deserialize, Serialize};

#[derive(Getters, Serialize, Deserialize, Debug)]
pub struct FourierTransform {
    io: NodeIo,
    id: NodeId,
    inverse: bool,
    real_output: bool,
}

impl HasNodeIo for FourierTransform {
    fn node_io(&self) -> &NodeIo {
        &self.io
    }
    fn node_io_mut(&mut self) -> &mut NodeIo {
        &mut self.io
    }
}

impl FourierTransform {
    pub fn new(inverse: bool, real_output: bool) -> Self {
        Self {
            io: NodeIo::new(),
            id: NodeId::new(),
            inverse,
            real_output,
        }
    }

    pub fn inverse(&self) -> bool {
        self.inverse
    }
    pub fn inverse_mut(&mut self) -> &mut bool {
        &mut self.inverse
    }
    pub fn real_output(&self) -> bool {
        self.real_output
    }
    pub fn real_output_mut(&mut self) -> &mut bool {
        &mut self.real_output
    }

    pub fn process_chunk(&self, chunk: DataChunk) -> DataChunk {
        let channels = *chunk.metadata().channels();
        let mut samples = match &chunk {
            DataChunk::Real(chunk) => (0..channels)
                .map(|c| {
                    chunk
                        .samples(c)
                        .iter()
                        .map(|s| Complex32::from_f32(*s).unwrap())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
            DataChunk::Complex(chunk) => (0..channels)
                .map(|c| chunk.samples(c).to_vec())
                .collect::<Vec<_>>(),
        };
        let mut transformed = samples.clone();
        let mut planner = FFTplanner::new(self.inverse);
        let fft = planner.plan_fft(*chunk.duration());
        for c in 0..channels {
            fft.process(&mut samples[c], &mut transformed[c]);
            let l = transformed[c].len();
            let normalize = Complex32::from_f32(l as f32).unwrap().sqrt();
            for s in transformed[c].iter_mut() {
                *s /= normalize;
            }
        }
        if self.real_output {
            DataChunk::Real(GenericDataChunk::new(
                transformed
                    .iter()
                    .map(|c| c.iter().map(|s| s.re).collect::<Vec<_>>())
                    .collect::<Vec<_>>(),
                chunk.metadata().clone(),
                *chunk.duration(),
                chunk.window_info().clone(),
            ))
        } else {
            DataChunk::Complex(GenericDataChunk::new(
                transformed,
                chunk.metadata().clone(),
                *chunk.duration(),
                chunk.window_info().clone(),
            ))
        }
    }
}

impl NodeTrait for FourierTransform {
    fn id(&self) -> NodeId {
        self.id
    }
    fn run_once(&mut self) {
        if self.inputs().len() != 1 {
            return;
        }
        while let Some(chunk) = self.inputs()[0].try_recv().ok() {
            let chunk = self.process_chunk(chunk);
            for output in self.outputs().iter() {
                let result = output.try_send(chunk.clone());
            }
        }
    }
}
