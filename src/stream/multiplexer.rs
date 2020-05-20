use crate::common::*;
use getset::Getters;
use rustfft::num_traits::Num;
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Getters)]
pub struct Multiplexer<S: Num + Clone> {
    #[getset(get = "pub")]
    receiver: Receiver<SampleChunk<S>>,
    senders: Vec<Option<Sender<SampleChunk<S>>>>,
}

impl<S: Num + Clone> Multiplexer<S> {
    pub fn new(receiver: Receiver<SampleChunk<S>>) -> Self {
        Self {
            receiver,
            senders: vec![],
        }
    }

    pub fn run(&mut self) {
        for chunk in self.receiver.iter() {
            for sender in self.senders.iter_mut().filter(|sender| sender.is_some()) {
                if let Err(_) = sender.as_ref().map(|s| s.send(chunk.clone())).unwrap() {
                    // discard dead senders
                    *sender = None
                }
            }
        }
    }

    pub fn new_receiver(&mut self) -> Receiver<SampleChunk<S>> {
        let (sender, receiver) = channel();
        self.senders.push(Some(sender));
        receiver
    }
}
