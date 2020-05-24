use crate::stream::node::{Event, EventReceiver, Runnable};
use getset::Getters;
use rodio;

#[derive(Getters)]
pub struct PlaybackSink {
    #[getset(get = "pub", set = "pub")]
    receiver: EventReceiver<f32>,
    #[getset(get = "pub")]
    rodio_sink: Option<rodio::Sink>,
}

impl PlaybackSink {
    pub fn new(receiver: EventReceiver<f32>) -> Self {
        Self {
            receiver,
            rodio_sink: None,
        }
    }

    pub fn play(&self) {
        if let Some(rodio_sink) = &self.rodio_sink {
            rodio_sink.play();
        }
    }

    pub fn pause(&self) {
        if let Some(rodio_sink) = &self.rodio_sink {
            rodio_sink.pause();
        }
    }

    pub fn set_rodio_sink(&mut self, rodio_sink: rodio::Sink) {
        let paused = if let Some(rodio_sink) = &self.rodio_sink {
            rodio_sink.is_paused()
        } else {
            false
        };
        if !paused {
            self.pause();
        }
        self.rodio_sink = Some(rodio_sink);
        if !paused {
            self.play();
        }
    }

    pub fn run_once(&mut self) -> bool {
        self.play();
        if let Ok(event) = self.receiver.recv() {
            match event {
                Event::Chunk(chunk) => {
                    if let Some(rodio_sink) = &self.rodio_sink {
                        rodio_sink.append(rodio::buffer::SamplesBuffer::new(
                            *chunk.metadata().channels() as u16,
                            *chunk.metadata().sample_rate() as u32,
                            chunk.flattened_samples(),
                        ))
                    }
                }
                Event::Stop => return true
            }
        }
        false
    }

    pub fn sleep_until_end(&self) {
        if let Some(rodio_sink) = &self.rodio_sink {
            rodio_sink.sleep_until_end();
        }
    }
}
