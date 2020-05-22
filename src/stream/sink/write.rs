use crate::stream::node::{Runnable, Event, EventReceiver};
use getset::Getters;
use hound;
use std::time;

#[derive(Getters)]
pub struct WriteSink {
    #[getset(get = "pub", set = "pub")]
    receiver: EventReceiver<f32>,
    #[getset(get = "pub")]
    filename: String,
    #[getset(get = "pub", set = "pub")]
    timeout: time::Duration,
}

impl WriteSink {
    pub fn new(receiver: EventReceiver<f32>, filename: String) -> Self {
        Self { receiver, filename, timeout: time::Duration::from_millis(10) }
    }
}

impl Runnable for WriteSink {
    fn run(&mut self) {
        let mut chunk = match self.receiver.recv().unwrap() {
            Event::Chunk(chunk) => chunk,
            _ => panic!("The first event was Stop"),
        };
        let spec = hound::WavSpec {
            channels: *chunk.metadata().channels() as u16,
            sample_rate: *chunk.metadata().sample_rate() as u32,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(&self.filename, spec).unwrap();
        loop {
            let samples = chunk.flattened_samples();
            for sample in samples {
                writer
                    .write_sample((sample * i16::MAX as f32) as i16)
                    .unwrap();
            }
            if let Ok(event) = self.receiver.recv_timeout(self.timeout) {
                chunk = match event {
                    Event::Chunk(chunk) => chunk,
                    Stop => break,
                };
            } else {
                break;
            }
        }
    }
}
