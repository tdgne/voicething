use crate::common::SampleChunk;
use rustfft::num_traits::Num;
use std::sync::mpsc::Receiver;

pub trait Node {
    fn run(&mut self);
}

pub trait SingleOutputNode<S: Num + Clone> {
    fn output(&mut self) -> Receiver<SampleChunk<S>>;
}

pub trait MultipleOutputNode<S: Num + Clone> {
    fn new_output(&mut self) -> Receiver<SampleChunk<S>>;
}
