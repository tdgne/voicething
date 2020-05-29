use crate::audio::common::{Sample, SampleChunk, WindowedSampleChunk, Chunk};
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender};
use rustfft::num_complex::Complex32;
use uuid::Uuid;

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

pub enum SingleInputNode {
    Real(Box<dyn SingleInput<f32, f32, SampleChunk<f32>, SampleChunk<f32>>>),
    Complex(Box<dyn SingleInput<Complex32, Complex32, SampleChunk<Complex32>, SampleChunk<Complex32>>>),
    WindowedReal(Box<dyn SingleInput<f32, f32, WindowedSampleChunk<f32>, WindowedSampleChunk<f32>>>),
    WindowedComplex(Box<dyn SingleInput<Complex32, Complex32, SampleChunk<Complex32>, SampleChunk<Complex32>>>),
    RealComplex(Box<dyn SingleInput<f32, Complex32, SampleChunk<f32>, SampleChunk<Complex32>>>),
    ComplexReal(Box<dyn SingleInput<Complex32, f32, SampleChunk<Complex32>, SampleChunk<f32>>>),
    WindowedRealComplex(Box<dyn SingleInput<f32, Complex32, WindowedSampleChunk<f32>, WindowedSampleChunk<Complex32>>>),
    WindowedComplexReal(Box<dyn SingleInput<Complex32, f32, SampleChunk<Complex32>, SampleChunk<f32>>>),
}

pub enum Node {
    SingleInput(SingleInputNode),
}

