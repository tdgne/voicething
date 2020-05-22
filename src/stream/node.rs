use crate::common::{Sample, SampleChunk};
use std::sync::mpsc::{Receiver, Sender};

#[derive(Clone)]
pub enum Event<S: Sample> {
    Chunk(SampleChunk<S>),
    Stop,
}

pub type EventSender<S: Sample> = Sender<Event<S>>;

pub type EventReceiver<S: Sample> = Receiver<Event<S>>;

pub trait Runnable: Send {
    fn run(&mut self);
}

pub trait ProcessNode<S: Sample> {
    fn receiver(&self) -> &EventReceiver<S>;

    fn sender(&self) -> Option<EventSender<S>>;

    fn process_chunk(&mut self, chunk: SampleChunk<S>) -> SampleChunk<S>;

    fn run(&mut self) {
        loop {
            let chunk = match self.receiver().recv() {
                Ok(Event::Chunk(chunk)) => chunk,
                Ok(Event::Stop) => {
                    if let Some(sender) = self.sender() {
                        sender.send(Event::Stop).unwrap()
                    };
                    continue
                },
                Err(_) => panic!("Error occurred during run()")
            };
            let chunk = self.process_chunk(chunk);
            if let Some(sender) = self.sender() {
                sender.send(Event::Chunk(chunk)).unwrap();
            }
        }
    }
}

pub trait SingleOutputNode<S: Sample>: Runnable {
    fn output(&mut self) -> EventReceiver<S>;
}

pub trait MultipleOutputNode<S: Sample>: Runnable {
    fn new_output(&mut self) -> EventReceiver<S>;
}
