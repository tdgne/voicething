use crate::stream::node::{Runnable, Event, EventReceiver};
use getset::Getters;
use rodio;

#[derive(Getters)]
pub struct PlaybackSink {
    #[getset(get = "pub", set = "pub")]
    receiver: EventReceiver<f32>,
    rodio_sink: rodio::Sink,
}

impl PlaybackSink {
    pub fn new(receiver: EventReceiver<f32>, rodio_sink: rodio::Sink) -> Self {
        Self {
            receiver,
            rodio_sink,
        }
    }
}

impl Runnable for PlaybackSink {
    fn run(&mut self) {
        use std::time::{Duration, SystemTime};
        let now = SystemTime::now();
        self.rodio_sink.play();
        println!("p {:?}", now.elapsed().unwrap());
        for event in self.receiver.iter() {
            match event {
                Event::Chunk(chunk) => self.rodio_sink.append(rodio::buffer::SamplesBuffer::new(
                    *chunk.metadata().channels() as u16,
                    *chunk.metadata().sample_rate() as u32,
                    chunk.flattened_samples(),
                )),
                Event::Stop => break,
            }
        }
        self.rodio_sink.sleep_until_end();
    }
}
