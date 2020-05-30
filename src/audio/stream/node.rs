use crate::audio::common::{Sample, SampleChunk, WindowedSampleChunk, Chunk};
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

pub type ChunkSender<S> = Sender<SampleChunk<S>>;

pub type SyncChunkSender<S> = SyncSender<SampleChunk<S>>;

pub type ChunkReceiver<S> = Receiver<SampleChunk<S>>;


pub fn chunk_channel<S: Sample>() -> (ChunkSender<S>, ChunkReceiver<S>) {
    channel()
}

pub fn sync_chunk_channel<S: Sample>(n: usize) -> (SyncChunkSender<S>, ChunkReceiver<S>) {
    sync_channel(n)
}

pub trait HasId {
    fn id(&self) -> Uuid;
}

pub trait SingleInput<S: Sample, T: Sample, I: Chunk<S>, O: Chunk<T>>: HasId {
    fn input(&self) -> Option<&Receiver<I>>;

    fn outputs(&self) -> &[SyncSender<O>];

    fn set_input(&mut self, rx: Option<Receiver<I>>);

    fn add_output(&mut self, tx: SyncSender<O>);

    fn process_chunk(&mut self, chunk: I) -> O;

    fn run_once(&mut self) {
        if let Some(input) = self.input() {
            if let Some(chunk) = input.try_recv().ok() {
                let chunk = self.process_chunk(chunk);
                for output in self.outputs().iter() {
                    let _ = output.try_send(chunk.clone());
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Node {
    Psola(super::psola::PsolaNode),
    Input(super::identity::IdentityNode<f32>),
    Output(super::identity::IdentityNode<f32>),
}

impl Node {
    pub fn id(&self) -> Uuid {
        use Node::*;
        match self {
            Psola(n) => n.id(),
            Input(n) => n.id(),
            Output(n) => n.id(),
        }
    }

    pub fn run_once(&mut self) {
    use Node::*;
        match self {
            Psola(n) => n.run_once(),
            Input(n) => n.run_once(),
            Output(n) => n.run_once(),
        }
    }

}
