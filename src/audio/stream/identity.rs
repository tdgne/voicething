use super::super::common::*;
use super::node::*;
use getset::Getters;
use uuid::Uuid;

#[derive(Getters)]
pub struct IdentityNode<S: Sample> {
    input: Option<ChunkReceiver<S>>,
    outputs: Vec<SyncChunkSender<S>>,
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

impl<S: Sample> SingleInput<S, S, SampleChunk<S>, SampleChunk<S>> for IdentityNode<S> {
    fn input(&self) -> Option<&ChunkReceiver<S>> {
        self.input.as_ref()
    }

    fn outputs(&self) -> &[SyncChunkSender<S>] {
        self.outputs.as_ref()
    }

    fn set_input(&mut self, rx: ChunkReceiver<S>) {
        self.input = Some(rx);
    }

    fn add_output(&mut self, tx: SyncChunkSender<S>) {
        self.outputs.push(tx);
    }

    fn process_chunk(&mut self, chunk: SampleChunk<S>) -> SampleChunk<S> {
        chunk
    }
}
