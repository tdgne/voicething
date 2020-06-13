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
    prev_unwrapped_phases: Vec<Vec<f32>>,
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
            prev_unwrapped_phases: vec![],
        }
    }

    pub fn rate(&self) -> &f32 {
        &self.rate
    }
    pub fn rate_mut(&mut self) -> &mut f32 {
        &mut self.rate
    }

    pub fn process_chunk(&mut self, chunk: SampleChunk) -> Option<SampleChunk> {
        let channels = *chunk.metadata().channels();
        while self.prev_unwrapped_phases.len() < channels {
            self.prev_unwrapped_phases.push(vec![]);
        }
        for p in self.prev_unwrapped_phases.iter_mut() {
            while p.len() < *chunk.duration_samples() {
                p.push(0.0);
            }
        }

        let mut incompatible = false;
        let samples = (0..channels).map(|c| match &chunk {
            SampleChunk::Real(_) => {
                eprintln!("incompatible input {}: {}", file!(), line!());
                incompatible = true;
                vec![]
            },
            SampleChunk::Complex(chunk) => chunk.samples(c).to_vec(),
        }).collect::<Vec<_>>();
        if incompatible {
            return None;
        }
        let mut unwrapped_phases = vec![vec![]; channels];
        for c in 0..channels {
            let duration = samples[c].len();
            for i in 0..duration {
                let mut prev_unwrapped_phase = self.prev_unwrapped_phases[c][i];
                if prev_unwrapped_phase.is_nan() {
                    prev_unwrapped_phase = 0.0;
                }
                let pi = 3.141592;
                let phase = samples[c][i].arg() % (2.0 * pi);
                let prev_phase = prev_unwrapped_phase % (2.0 * pi);
                let unwrapped_phase = prev_unwrapped_phase + phase - prev_phase + if phase - prev_phase < -pi {
                    2.0 * pi
                } else if phase - prev_phase > pi {
                    - 2.0 * pi
                } else {
                    0.0
                };
                unwrapped_phases[c].push(unwrapped_phase);
            }
        }

        let mut scaled = vec![vec![]; channels];
        for c in 0..channels {
            let duration = samples[c].len();
            for i in 0..duration/2 {
                let unscaled_index = (i as f32 / self.rate).ceil() as usize;
                if unscaled_index < duration/2 {
                    scaled[c].push(Complex32::from_polar(&samples[c][unscaled_index].norm(), &(self.rate * unwrapped_phases[c][unscaled_index])));
                } else {
                    scaled[c].push(Complex32::zero());
                }
            }
            for i in duration/2..duration {
                let unscaled_index = duration - ((duration - i - 1) as f32 / self.rate).ceil() as usize - 1;
                if unscaled_index >= duration/2 {
                    scaled[c].push(Complex32::from_polar(&samples[c][unscaled_index].norm(), &(self.rate * unwrapped_phases[c][unscaled_index])));
                } else {
                    scaled[c].push(Complex32::zero());
                }
            }
        }

        self.prev_unwrapped_phases = unwrapped_phases;

        let new_chunk = GenericSampleChunk::new(
            scaled,
            chunk.metadata().clone(),
            chunk.duration_samples().clone(),
            chunk.window_info().clone(),
        );
        Some(SampleChunk::Complex(new_chunk))
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
            if let Some(chunk) = self.process_chunk(chunk) {
                for output in self.outputs().iter() {
                    let result = output.try_send(chunk.clone());
                }
            }
        }
    }
}
