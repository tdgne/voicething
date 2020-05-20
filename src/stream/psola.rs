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
                let channels = *chunk.metadata().channels();
                let mut new_samples = vec![];
                for channel in 0..channels {
                    new_samples.push(psola(chunk.samples(channel)));
                }
                let new_chunk = SampleChunk::new(
                    new_samples,
                    chunk.metadata().clone(),
                    *chunk.duration_samples(),
                );
                sender.send(new_chunk).expect("channel broken");
            }
        }
    }

    pub fn output(&mut self) -> Receiver<SampleChunk<f32>> {
        let (sender, receiver) = channel();
        self.sender = Some(sender);
        receiver
    }
}

fn psola(data: &[f32]) -> Vec<f32> {
    data.to_vec()
}
