use super::super::common::*;
use super::node::*;
use getset::Getters;
use rustfft::num_complex::Complex32;
use rustfft::num_traits::*;
use serde::{Deserialize, Serialize};

#[derive(Getters, Serialize, Deserialize, Debug)]
pub struct PhaseVocoder {
    io: NodeIo,
    id: NodeId,
    rate: f32,
}

impl HasNodeIo for PhaseVocoder {
    fn node_io(&self) -> &NodeIo {
        &self.io
    }
    fn node_io_mut(&mut self) -> &mut NodeIo {
        &mut self.io
    }
}

impl PhaseVocoder {
    pub fn new(rate: f32) -> Self {
        Self {
            io: NodeIo::new(),
            id: NodeId::new(),
            rate,
        }
    }

    pub fn rate(&self) -> &f32 {
        &self.rate
    }
    pub fn rate_mut(&mut self) -> &mut f32 {
        &mut self.rate
    }

    pub fn process_chunk(&self, chunk: SampleChunk) -> SampleChunk {
        let channels = *chunk.metadata().channels();
        let samples = (0..channels).map(|c| match &chunk {
            SampleChunk::Real(chunk) => chunk
                .samples(c)
                .iter()
                .map(|s| Complex32::from_f32(*s).unwrap())
                .collect::<Vec<_>>(),
            SampleChunk::Complex(chunk) => chunk.samples(c).to_vec(),
        }).collect::<Vec<_>>();
        let mut scaled = vec![vec![]; channels];
        for c in 0..channels {
            let duration = samples[c].len();
            for i in 0..duration/2 {
                let unscaled_index = (i as f32 / self.rate).round() as usize;
                scaled[c].push(samples[c][unscaled_index])
            }
            for i in 0..duration/2 {
                let mirrored_value = scaled[c][duration/2-i-1];
                scaled[c].push(mirrored_value.conj());
            }
        }
        match &chunk {
            SampleChunk::Real(_) => chunk,
            SampleChunk::Complex(_) => {
                let new_chunk = GenericSampleChunk::new(
                    scaled,
                    chunk.metadata().clone(),
                    chunk.duration_samples().clone(),
                    chunk.window_info().clone(),
                );
                SampleChunk::Complex(new_chunk)
            }
        }
    }
}

impl NodeTrait for PhaseVocoder {
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
