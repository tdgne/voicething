use crate::audio::common::{Sample, SampleChunk, WindowedSampleChunk, Chunk};
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender};
use rustfft::num_complex::Complex32;
use uuid::Uuid;
use std::sync::{Arc, Mutex};

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

    fn outputs(&self) -> &[Sender<O>];

    fn set_input(&mut self, rx: Receiver<I>);

    fn add_output(&mut self, tx: Sender<O>);

    fn process_chunk(&mut self, chunk: I) -> O;

    fn run_once(&mut self) {
        if let Some(input) = self.input() {
            if let Some(chunk) = input.try_recv().ok() {
                let chunk = self.process_chunk(chunk);
                for output in self.outputs().iter() {
                    let _ = output.send(chunk.clone());
                }
            }
        }
    }
}

type NodeWrapper<N> = Arc<Mutex<N>>;

pub enum Node {
    Psola(NodeWrapper<super::psola::PsolaNode>),
}

pub enum NodeIoType {
    Real,
    Complex,
    WindowedReal,
    WindowedComplex,
}

impl Node {
    pub fn io_type(&self) -> NodeIoType {
        use Node::*;
        use NodeIoType::*;
        match self {
            Psola(_) => Real,
        }
    }
}
