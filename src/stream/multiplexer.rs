use crate::common::*;
use crate::stream::node::{EventReceiver, EventSender};
use crate::stream::node::{MultipleOutputNode, Runnable};
use getset::Getters;
use std::sync::mpsc::channel;

#[derive(Getters)]
pub struct Multiplexer<S: Sample> {
    receiver: EventReceiver<S>,
    senders: Vec<Option<EventSender<S>>>,
}

impl<S: Sample> MultipleOutputNode<S> for Multiplexer<S> {
    fn new_output(&mut self) -> EventReceiver<S> {
        let (sender, receiver) = channel();
        self.senders.push(Some(sender));
        receiver
    }
}

impl<S: Sample> Multiplexer<S> {
    pub fn new(receiver: EventReceiver<S>) -> Self {
        Self {
            receiver,
            senders: vec![],
        }
    }
}

impl<S: Sample> Runnable for Multiplexer<S> {
    fn run(&mut self) {
        for event in self.receiver.iter() {
            for sender in self.senders.iter_mut().filter(|sender| sender.is_some()) {
                if let Err(_) = sender.as_ref().map(|s| s.send(event.clone())).unwrap() {
                    // discard dead senders
                    *sender = None
                }
            }
        }
    }
}
