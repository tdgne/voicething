use crate::audio::common::*;
use crate::audio::stream::node::{EventReceiver, EventSender};
use crate::audio::stream::node::{MultipleOutputNode, Runnable};
use getset::Getters;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};

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

    pub fn run_once(&mut self) {
        let event = self.receiver.recv().unwrap();
        for sender in self.senders.iter_mut().filter(|sender| sender.is_some()) {
            if let Err(_) = sender.as_ref().map(|s| s.send(event.clone())).unwrap() {
                // discard dead senders
                *sender = None
            }
        }
    }

    pub fn set_receiver(&mut self, receiver: EventReceiver<S>) {
        self.receiver = receiver;
    }
}

impl<S: Sample> Runnable for Multiplexer<S> {
    fn run(&mut self) {
        loop {
            self.run_once();
        }
    }
}
