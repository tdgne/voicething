use crate::common::{Sample, SampleChunk};
use std::sync::mpsc::Receiver;

pub trait Node: Send {
    fn run(&mut self);
}

pub trait SingleOutputNode<S: Sample>: Node {
    fn output(&mut self) -> Receiver<SampleChunk<S>>;
}

pub trait MultipleOutputNode<S: Sample>: Node {
    fn new_output(&mut self) -> Receiver<SampleChunk<S>>;
}
