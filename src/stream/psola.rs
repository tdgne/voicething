use crate::common::*;
use getset::Getters;
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Getters)]
pub struct PsolaNode {
    #[getset(get = "pub", set = "pub")]
    receiver: Receiver<SampleChunk<f32>>,
    sender: Option<Sender<SampleChunk<f32>>>,
}

impl PsolaNode {
    pub fn new(receiver: Receiver<SampleChunk<f32>>) -> Self {
        Self {
            receiver,
            sender: None,
        }
    }

    pub fn run(&mut self) {
        for chunk in self.receiver.iter() {
            if let Some(ref mut sender) = self.sender {
                sender.send(psola(chunk));
            }
        }
    }

    pub fn output(&mut self) -> Receiver<SampleChunk<f32>> {
        let (sender, receiver) = channel();
        self.sender = Some(sender);
        receiver
    }
}

fn psola(chunk: SampleChunk<f32>) -> SampleChunk<f32> {
    unimplemented!()
}
