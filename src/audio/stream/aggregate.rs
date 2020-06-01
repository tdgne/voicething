use super::super::common::*;
use super::node::*;
use getset::Getters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub enum AggregateSetting<S: Sample> {
    LinearCombination(Vec<S>),
    Product,
}

#[derive(Getters, Serialize, Deserialize, Debug)]
pub struct AggregateNode<S: Sample> {
    #[serde(skip)]
    inputs: Vec<ChunkReceiver<S>>,
    #[serde(skip)]
    outputs: Vec<SyncChunkSender<S>>,
    #[getset(get = "pub")]
    setting: AggregateSetting<S>,
    id: Uuid,
}

impl<S: Sample> HasId for AggregateNode<S> {
    fn id(&self) -> Uuid {
        self.id
    }
}

impl<S: Sample> AggregateNode<S> {
    pub fn new(setting: AggregateSetting<S>) -> Self {
        Self {
            inputs: vec![],
            outputs: vec![],
            id: Uuid::new_v4(),
            setting,
        }
    }

    pub fn inputs(&self) -> &[ChunkReceiver<S>] {
        self.inputs.as_ref()
    }

    pub fn outputs(&self) -> &[SyncChunkSender<S>] {
        self.outputs.as_ref()
    }

    pub fn add_input(&mut self, rx: ChunkReceiver<S>) {
        self.inputs.push(rx);
    }

    pub fn add_output(&mut self, tx: SyncChunkSender<S>) {
        self.outputs.push(tx);
    }

    pub fn process_chunk(&mut self, chunks: Vec<SampleChunk<S>>) -> Option<SampleChunk<S>> {
        if chunks.len() != self.inputs.len() || chunks.len() == 0 {
            return None;
        }
        let samples = chunks
            .iter()
            .map(|c| c.flattened_samples())
            .collect::<Vec<_>>();
        let l = samples[0].len();
        let new_samples = match &self.setting {
            AggregateSetting::LinearCombination(coefs) => (0..l)
                .map(|i| {
                    coefs
                        .iter()
                        .enumerate()
                        .map(|(k, coef)| *coef * samples[k][i])
                        .fold(S::from_f32(0.0).unwrap(), |acc, cur| acc + cur)
                })
                .collect::<Vec<_>>(),
            AggregateSetting::Product => (0..l)
                .map(|i| {
                    (0..samples.len())
                        .map(|k| samples[k][i])
                        .fold(S::from_f32(1.0).unwrap(), |acc, cur| acc * cur)
                })
                .collect::<Vec<_>>(),
        };
        SampleChunk::from_flat_samples(&new_samples, chunks[0].metadata().clone()).ok()
    }

    pub fn run_once(&mut self) {
        let chunks = self
            .inputs
            .iter()
            .flat_map(|input| input.try_recv().ok())
            .collect::<Vec<_>>();
        let chunk = self.process_chunk(chunks).unwrap();
        for output in self.outputs().iter() {
            let _ = output.try_send(chunk.clone());
        }
    }
}
