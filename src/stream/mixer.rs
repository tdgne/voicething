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
    output_chunk_duration: usize,
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
    let mut out_chunk = SampleChunk::from_flat_samples(
        &vec![S::zero(); chunk.metadata().channels() * chunk.duration_samples()],
        out_format,
    )
    .unwrap();
    let out_duration = (*chunk.duration_samples() as f32 / *chunk.metadata().sample_rate() as f32
        * out_sample_rate as f32) as usize;
    let channels = *chunk.metadata().channels();
    for c in 0..channels {
        let in_samples = chunk.samples(c);
        let out_samples = out_chunk.samples_mut(c);
        for i in 0..out_duration {
            out_samples[i] = in_samples[(i as f32 / out_sample_rate as f32
                * *chunk.metadata().sample_rate() as f32)
                as usize]
        }
    }
    out_chunk
}

impl<S: Sample> Runnable for Mixer<S> {
    fn run(&mut self) {
        let out_channels = *self.output_format.channels();
        let out_sample_rate = *self.output_format.sample_rate();
        'outer: loop {
            let mut mixed_chunk = SampleChunk::from_flat_samples(
                &vec![S::zero(); out_channels * self.output_chunk_duration],
                self.output_format.clone(),
            )
            .unwrap();
            for rvp in self.receivers.iter().flat_map(|rvp| rvp.iter()) {
                let volume = rvp.volume;
                let chunk = match rvp.receiver.recv() {
                    Ok(event) => match event {
                        Event::Chunk(chunk) => chunk,
                        Event::Stop => {
                            if let Some(ref sender) = self.sender {
                                sender.send(Event::Stop).unwrap();
                            }
                            break 'outer;
                        }
                    },
                    Err(_) => panic!("An error occurred in Mixer"),
                };

                let formatted_chunk = format_chunk_sample_rate(
                    format_chunk_channel(chunk, out_channels),
                    out_sample_rate,
                );
                for c in 0..out_channels {
                    let mixed_samples = mixed_chunk.samples_mut(c);
                    let formatted_samples = formatted_chunk.samples(c);
                    for i in 0..self.output_chunk_duration {
                        mixed_samples[i] = formatted_samples[i] * S::from_f32(volume).unwrap();
                    }
                }
            }
            if let Some(ref sender) = self.sender {
                sender.send(Event::Chunk(mixed_chunk)).unwrap();
            }
        }
    }
}

impl<S: Sample> SingleOutputNode<S> for Mixer<S> {
    fn output(&mut self) -> EventReceiver<S> {
        let (sender, receiver) = channel();
        self.sender = Some(sender);
        receiver
    }
}

impl<S: Sample> Mixer<S> {
    pub fn new(
        receivers: Vec<ReceiverVolumePair<S>>,
        output_format: AudioMetadata,
        output_chunk_duration: usize,
    ) -> Self {
        Self {
            receivers: receivers.into_iter().map(|r| Some(r)).collect::<Vec<_>>(),
            sender: None,
            output_format,
            output_chunk_duration,
        }
    }

    pub fn add_input(&mut self, rvp: ReceiverVolumePair<S>) {
        self.receivers.push(Some(rvp));
    }
}
