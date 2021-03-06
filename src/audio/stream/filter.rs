use super::super::common::*;
use super::node::*;
use getset::Getters;
use rustfft::num_complex::Complex32;
use rustfft::num_traits::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
pub enum FilterOperation {
    ReplaceLowerAmplitudesFd { value: f32, threshold: f32 },
    ReplaceHigherAmplitudesFd { value: f32, threshold: f32 },
    ReplaceLowerAmplitudesTd { value: f32, threshold: usize },
    ReplaceHigherAmplitudesTd { value: f32, threshold: usize },
}

#[derive(Getters, Serialize, Deserialize, Debug)]
pub struct FilterNode {
    io: NodeIo,
    id: NodeId,
    op: FilterOperation,
}

impl HasNodeIo for FilterNode {
    fn node_io(&self) -> &NodeIo {
        &self.io
    }
    fn node_io_mut(&mut self) -> &mut NodeIo {
        &mut self.io
    }
}

impl FilterNode {
    pub fn new(op: FilterOperation) -> Self {
        Self {
            io: NodeIo::new(),
            id: NodeId::new(),
            op,
        }
    }

    pub fn op(&self) -> &FilterOperation {
        &self.op
    }
    pub fn op_mut(&mut self) -> &mut FilterOperation {
        &mut self.op
    }

    pub fn process_chunk(&self, chunk: DataChunk) -> DataChunk {
        let channels = *chunk.metadata().channels();
        let samples = (0..channels)
            .map(|c| match &chunk {
                DataChunk::Real(chunk) => chunk
                    .samples(c)
                    .iter()
                    .map(|s| Complex32::from_f32(*s).unwrap())
                    .collect::<Vec<_>>(),
                DataChunk::Complex(chunk) => chunk.samples(c).to_vec(),
            })
            .map(|c| {
                c.iter()
                    .enumerate()
                    .map(|(i, s)| match self.op {
                        FilterOperation::ReplaceLowerAmplitudesFd { value, threshold } => {
                            let sample_rate = *chunk.metadata().sample_rate() as f32;
                            let chunk_duration = *chunk.duration() as f32;
                            let threshold = (threshold / (sample_rate / chunk_duration)) as usize;
                            if threshold < (chunk_duration / 2.0) as usize {
                                if i < threshold {
                                    Complex32::from_f32(value).unwrap() * s
                                } else if i < chunk_duration as usize - threshold {
                                    s.clone()
                                } else {
                                    Complex32::from_f32(value).unwrap() * s
                                }
                            } else {
                                s.clone()
                            }
                        }
                        FilterOperation::ReplaceHigherAmplitudesFd { value, threshold } => {
                            let sample_rate = *chunk.metadata().sample_rate() as f32;
                            let chunk_duration = *chunk.duration() as f32;
                            let threshold = (threshold / (sample_rate / chunk_duration)) as usize;
                            if threshold < (chunk_duration / 2.0) as usize {
                                if i < threshold {
                                    s.clone()
                                } else if i < chunk_duration as usize - threshold {
                                    Complex32::from_f32(value).unwrap() * s
                                } else {
                                    s.clone()
                                }
                            } else {
                                s.clone()
                            }
                        }
                        FilterOperation::ReplaceLowerAmplitudesTd { value, threshold } => {
                            let chunk_duration = *chunk.duration();
                            if threshold < chunk_duration / 2 {
                                if i < threshold {
                                    Complex32::from_f32(value).unwrap() * s
                                } else if i < chunk_duration as usize - threshold {
                                    s.clone()
                                } else {
                                    Complex32::from_f32(value).unwrap() * s
                                }
                            } else {
                                s.clone()
                            }
                        }

                        FilterOperation::ReplaceHigherAmplitudesTd { value, threshold } => {
                            let chunk_duration = *chunk.duration();
                            if threshold < chunk_duration / 2 {
                                if i < threshold {
                                    s.clone()
                                } else if i < chunk_duration as usize - threshold {
                                    Complex32::from_f32(value).unwrap() * s
                                } else {
                                    s.clone()
                                }
                            } else {
                                s.clone()
                            }
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        match &chunk {
            DataChunk::Real(_) => chunk,
            DataChunk::Complex(_) => {
                let new_chunk = GenericDataChunk::new(
                    samples,
                    chunk.metadata().clone(),
                    chunk.duration().clone(),
                    chunk.window_info().clone(),
                );
                DataChunk::Complex(new_chunk)
            }
        }
    }
}

impl NodeTrait for FilterNode {
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
