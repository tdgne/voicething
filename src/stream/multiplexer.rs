use crate::common::*;
use crate::stream::node::{Node, MultipleOutputNode};
use getset::Getters;
use rustfft::num_traits::Num;
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Getters)]
pub struct Multiplexer<S: Sample> {
    receiver: Receiver<SampleChunk<S>>,
    senders: Vec<Option<Sender<SampleChunk<S>>>>,
}

impl<S: Sample> Node for Multiplexer<S> {
    fn run(&mut self) {
        for chunk in self.receiver.iter() {
            for sender in self.senders.iter_mut().filter(|sender| sender.is_some()) {
                if let Err(_) = sender.as_ref().map(|s| s.send(chunk.clone())).unwrap() {
                    // discard dead senders
                    *sender = None
                }
            }
        }
    }
}

impl<S: Sample> MultipleOutputNode<S> for Multiplexer<S> {
    fn new_output(&mut self) -> Receiver<SampleChunk<S>> {
        let (sender, receiver) = channel();
        self.senders.push(Some(sender));
        receiver
    }
}

impl<S: Sample> Multiplexer<S> {
    pub fn new(receiver: Receiver<SampleChunk<S>>) -> Self {
        Self {
            receiver,
            senders: vec![],
        }
    }
}
