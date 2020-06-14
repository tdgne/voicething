use super::super::common::*;
use super::node::*;
use super::psola::PsolaNode;
use getset::Getters;
use serde::{Deserialize, Serialize};

#[derive(Getters, Serialize, Deserialize, Debug)]
pub struct PeriodReplicator {
    io: NodeIo,
    #[serde(skip)]
    wave: Option<Vec<f32>>,
    #[serde(skip)]
    last_chunk: Option<SampleChunk>,
    #[serde(skip)]
    phase: usize,
    id: NodeId,
}

impl HasNodeIo for PeriodReplicator {
    fn node_io(&self) -> &NodeIo {
        &self.io
    }
    fn node_io_mut(&mut self) -> &mut NodeIo {
        &mut self.io
    }
}

impl NodeTrait for PeriodReplicator {
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
                    let _ = output.try_send(chunk.clone());
                }
            }
        }
    }
}

impl PeriodReplicator {
    pub fn new() -> Self {
        Self {
            io: NodeIo::new(),
            id: NodeId::new(),
            wave: None,
            last_chunk: None,
            phase: 0,
        }
    }

    pub fn wave(&self) -> Option<&Vec<f32>> {
        self.wave.as_ref()
    }

    pub fn grab_period(&mut self) -> bool {
        if let Some(chunk) = &self.last_chunk {
            let data = match chunk {
                SampleChunk::Real(chunk) => chunk.samples(0),
                _ => unreachable!(),
            };
            if let Some(period) = PsolaNode::period(
                data,
                (50, 800),
                data.iter().fold(0.0, |a, b| f32::max(a * a, b * b)) / 4.0,
            ) {
                self.wave = Some(
                    data.iter()
                        .skip((data.len() / 2 - period / 2).max(0))
                        .take(period)
                        .cloned()
                        .collect::<Vec<_>>(),
                );
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn discard_period(&mut self) -> bool {
        if self.wave.is_some() {
            self.wave = None;
            true
        } else {
            false
        }
    }

    fn process_chunk(&mut self, chunk: SampleChunk) -> Option<SampleChunk> {
        match chunk {
            SampleChunk::Real(_) => self.last_chunk = Some(chunk.clone()),
            _ => {
                eprintln!("incompatible input {}: {}", file!(), line!());
                return None;
            }
        };
        if let Some(wave) = &self.wave {
            let samples = wave
                .iter()
                .cycle()
                .skip(self.phase)
                .take(*chunk.duration_samples())
                .cloned()
                .collect::<Vec<_>>();
            self.phase += *chunk.duration_samples() % wave.len();
            self.phase %= wave.len();
            Some(SampleChunk::Real(GenericSampleChunk::new(
                vec![samples; *chunk.metadata().channels()],
                chunk.metadata().clone(),
                *chunk.duration_samples(),
                chunk.window_info().clone(),
            )))
        } else {
            Some(chunk)
        }
    }
}
