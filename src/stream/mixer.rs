use crate::common::*;
use crate::stream::node::{Event, EventReceiver, EventSender};
use crate::stream::node::{Runnable, SingleOutputNode};
use getset::Getters;
use std::sync::mpsc::channel;

pub struct ReceiverVolumePair<S: Sample> {
    pub receiver: EventReceiver<S>,
    pub volume: f32,
}

#[derive(Getters)]
pub struct Mixer<S: Sample> {
    #[getset(get = "pub")]
    receivers: Vec<Option<ReceiverVolumePair<S>>>,
    sender: Option<EventSender<S>>,
    #[getset(get = "pub", set = "pub")]
    output_format: AudioMetadata,
    #[getset(get = "pub", set = "pub")]
    max_output_chunk_duration: usize,
    buffer: Vec<S>,
}

fn format_chunk_channel<S: Sample>(chunk: SampleChunk<S>, out_channels: usize) -> SampleChunk<S> {
    let out_format = AudioMetadata::new(out_channels, *chunk.metadata().sample_rate());
    let mut out_chunk = SampleChunk::from_flat_samples(
        &vec![S::zero(); out_channels * chunk.duration_samples()],
        out_format,
    )
    .unwrap();
    let chunk_channels = *chunk.metadata().channels();
    match out_channels {
        1 => {
            for c in 0..chunk_channels {
                for (i, out_chunk_samples) in out_chunk.samples_mut(c).iter_mut().enumerate() {
                    *out_chunk_samples +=
                        chunk.samples(0)[i] * S::from_f32(1.0 / chunk_channels as f32).unwrap();
                }
            }
        }
        2 => {
            if chunk_channels == 1 {
                for (i, sample) in chunk.samples(0).iter().enumerate() {
                    out_chunk.samples_mut(0)[i] = *sample * S::from_f32(1.0).unwrap();
                    out_chunk.samples_mut(1)[i] = *sample * S::from_f32(1.0).unwrap();
                }
            } else if chunk_channels == 2 {
                for c in 0..chunk_channels {
                    for (i, sample) in chunk.samples(c).iter().enumerate() {
                        out_chunk.samples_mut(c)[i] = *sample * S::from_f32(1.0).unwrap();
                    }
                }
            } else {
                unimplemented!()
            }
        }
        _ => unimplemented!(),
    }
    out_chunk
}

fn format_chunk_sample_rate<S: Sample>(
    chunk: SampleChunk<S>,
    out_sample_rate: usize,
) -> SampleChunk<S> {
    if out_sample_rate == *chunk.metadata().sample_rate() {
        return chunk;
    }
    let out_format = AudioMetadata::new(*chunk.metadata().channels(), out_sample_rate);
    let out_duration = (*chunk.duration_samples() as f32 / *chunk.metadata().sample_rate() as f32
        * out_sample_rate as f32)
        .floor() as usize;
    let mut out_chunk = SampleChunk::from_flat_samples(
        &vec![S::zero(); chunk.metadata().channels() * out_duration],
        out_format,
    )
    .unwrap();
    let channels = *chunk.metadata().channels();
    for c in 0..channels {
        let in_samples = chunk.samples(c);
        let out_samples = out_chunk.samples_mut(c);
        for i in 0..out_duration {
            out_samples[i] = in_samples[(i as f32 / out_sample_rate as f32
                * *chunk.metadata().sample_rate() as f32)
                .floor() as usize]
        }
    }
    out_chunk
}

impl<S: Sample> Mixer<S> {
    pub fn run(&mut self) {
        loop {
            self.run_once(1024);
        }
    }

    pub fn run_once(&mut self, out_duration: usize) {
        let out_channels = *self.output_format.channels();
        let out_sample_rate = *self.output_format.sample_rate();
        while self.buffer.len() < out_duration * out_channels {
            let mut mixed_chunk = SampleChunk::from_flat_samples(
                &vec![S::zero(); out_channels * self.max_output_chunk_duration],
                self.output_format.clone(),
            )
            .unwrap();
            let mut duration = None;
            for rvp in self.receivers.iter().flat_map(|rvp| rvp.iter()) {
                let volume = rvp.volume;
                let chunk = match rvp.receiver.recv() {
                    Ok(event) => match event {
                        Event::Chunk(chunk) => chunk,
                        Event::Stop => {
                            if let Some(ref sender) = self.sender {
                                sender.send(Event::Stop).unwrap();
                            }
                            return;
                        }
                    },
                    Err(_) => panic!("An error occurred in Mixer"),
                };

                let formatted_chunk = format_chunk_sample_rate(
                    format_chunk_channel(chunk, out_channels),
                    out_sample_rate,
                );

                if let Some(duration) = duration {
                    if duration != *formatted_chunk.duration_samples() {
                        panic!("Input chunks have different durations.");
                    }
                } else {
                    duration = Some(*formatted_chunk.duration_samples());
                }

                for c in 0..out_channels {
                    let mixed_samples = mixed_chunk.samples_mut(c);
                    let formatted_samples = formatted_chunk.samples(c);
                    for i in 0..formatted_samples.len() {
                        mixed_samples[i] = formatted_samples[i] * S::from_f32(volume).unwrap();
                    }
                }
            }

            if let Some(duration) = duration {
                mixed_chunk.truncate(duration);
            }

            self.buffer.append(&mut mixed_chunk.flattened_samples());
        }

        let chunk = SampleChunk::from_flat_samples(
            &self.buffer[0..out_duration * out_channels],
            AudioMetadata::new(out_channels, out_sample_rate),
        )
        .unwrap();

        if let Some(ref sender) = self.sender {
            sender.send(Event::Chunk(chunk)).unwrap();
        }
    }

    pub fn output(&mut self) -> EventReceiver<S> {
        let (sender, receiver) = channel();
        self.sender = Some(sender);
        receiver
    }

    pub fn add_receiver(&mut self, receiver: EventReceiver<S>, volume: f32) {
        self.receivers.push(Some(ReceiverVolumePair{receiver, volume}));
    }

    pub fn new(
        receivers: Vec<ReceiverVolumePair<S>>,
        output_format: AudioMetadata,
        max_output_chunk_duration: usize,
    ) -> Self {
        Self {
            receivers: receivers.into_iter().map(|r| Some(r)).collect::<Vec<_>>(),
            sender: None,
            output_format,
            max_output_chunk_duration, // TODO: stop relying on max_output_chunk_duration
            buffer: vec![],
        }
    }

    pub fn add_input(&mut self, rvp: ReceiverVolumePair<S>) {
        self.receivers.push(Some(rvp));
    }
}
