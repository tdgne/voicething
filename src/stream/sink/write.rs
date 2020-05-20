use crate::common::*;
use getset::Getters;
use std::sync::mpsc::Receiver;
use std::time;
use hound;

#[derive(Getters)]
pub struct WriteSink {
    #[getset(get = "pub", set = "pub")]
    receiver: Receiver<SampleChunk<f32>>,
    #[getset(get = "pub")]
    filename: String,
}

impl WriteSink {
    pub fn new(receiver: Receiver<SampleChunk<f32>>, filename: String) -> Self {
        Self {
            receiver, filename
        }
    }

    pub fn run(&self, timeout: time::Duration) {
        let mut chunk = self.receiver.recv().unwrap();
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
                writer.write_sample((sample * i16::MAX as f32) as i16).unwrap();
            }
            if let Ok(next_chunk) = self.receiver.recv_timeout(timeout) {
                chunk = next_chunk;
            } else {
                break;
            }
        }
    }
}
