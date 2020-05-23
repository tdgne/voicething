use crate::common::{Sample, SampleChunk};
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Clone)]
pub enum Event<S: Sample> {
    Chunk(SampleChunk<S>),
    Stop,
}

pub type EventSender<S> = Sender<Event<S>>;

pub type EventReceiver<S> = Receiver<Event<S>>;

pub fn event_channel<S: Sample>() -> (EventSender<S>, EventReceiver<S>) {
    channel()
}

pub trait Runnable: Send {
    fn run(&mut self);
}

pub trait ProcessNode<S: Sample> {
    fn receiver(&self) -> &EventReceiver<S>;

    fn sender(&self) -> Option<EventSender<S>>;

    fn process_chunk(&mut self, chunk: SampleChunk<S>) -> SampleChunk<S>;

    fn run_once(&mut self) {
        let chunk = match self.receiver().recv() {
            Ok(Event::Chunk(chunk)) => chunk,
            Ok(Event::Stop) => {
                if let Some(sender) = self.sender() {
                    sender.send(Event::Stop).unwrap();
                }
                return
            },
            Err(_) => panic!("Error occurred during run()")
        };
        let chunk = self.process_chunk(chunk);
        if let Some(sender) = self.sender() {
            sender.send(Event::Chunk(chunk)).unwrap();
        }
    }
}

pub trait SingleOutputNode<S: Sample>: Runnable {
    fn output(&mut self) -> EventReceiver<S>;
}

pub trait MultipleOutputNode<S: Sample>: Runnable {
    fn new_output(&mut self) -> EventReceiver<S>;
}
