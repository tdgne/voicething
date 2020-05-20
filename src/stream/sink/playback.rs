use crate::common::*;
use getset::Getters;
use rodio;
use std::sync::mpsc::Receiver;

#[derive(Getters)]
pub struct PlaybackSink {
    #[getset(get = "pub", set = "pub")]
    receiver: Receiver<SampleChunk<f32>>,
    rodio_sink: rodio::Sink,
}

impl PlaybackSink {
    pub fn new(receiver: Receiver<SampleChunk<f32>>, rodio_sink: rodio::Sink) -> Self {
        Self {
            receiver,
            rodio_sink,
        }
    }

    pub fn start_playback(&self) {
        self.rodio_sink.play();
        for chunk in self.receiver.iter() {
            self.rodio_sink.append(rodio::buffer::SamplesBuffer::new(
                *chunk.metadata().channels() as u16,
                *chunk.metadata().sample_rate() as u32,
                chunk.flattened_samples(),
            ))
        }
    }
}
