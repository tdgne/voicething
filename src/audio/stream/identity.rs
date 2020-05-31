use super::super::common::*;
use super::node::*;
use getset::Getters;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Getters, Serialize, Deserialize, Debug)]
pub struct IdentityNode<S: Sample> {
    #[serde(skip)]
    input: Option<ChunkReceiver<S>>,
    #[serde(skip)]
    outputs: Vec<SyncChunkSender<S>>,
    #[serde(skip)]
    #[getset(get = "pub")]
    name: String,
    id: Uuid,
}

impl<S: Sample> HasId for IdentityNode<S> {
    fn id(&self) -> Uuid {
        self.id
    }
}

impl<S: Sample> IdentityNode<S> {
    pub fn new(name: String) -> Self {
        Self {
            input: None,
            outputs: vec![],
            name,
            id: Uuid::new_v4(),
        }
    }
}

impl<S: Sample> SingleInput<S, S> for IdentityNode<S> {
    fn input(&self) -> Option<&ChunkReceiver<S>> {
        self.input.as_ref()
    }

    fn outputs(&self) -> &[SyncChunkSender<S>] {
        self.outputs.as_ref()
    }

    fn set_input(&mut self, rx: Option<ChunkReceiver<S>>) {
        self.input = rx;
    }

    fn add_output(&mut self, tx: SyncChunkSender<S>) {
        self.outputs.push(tx);
    }

    fn process_chunk(&mut self, chunk: SampleChunk<S>) -> SampleChunk<S> {
        chunk
    }
}
