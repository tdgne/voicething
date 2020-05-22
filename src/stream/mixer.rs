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

impl<S: Sample> Runnable for Mixer<S> {
    fn run(&mut self) {
        // TODO: support different sample rates and chunk durations
        // TODO: make re-chunker and resampler nodes and make the mixer a compound node
        let out_channels = *self.output_format.channels();
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

                let chunk_channels = *chunk.metadata().channels();
                match out_channels {
                    1 => {
                        for c in 0..chunk_channels {
                            for (i, mixed_chunk_samples) in
                                mixed_chunk.samples_mut(c).iter_mut().enumerate()
                            {
                                *mixed_chunk_samples += chunk.samples(0)[i]
                                    * S::from_f32(volume / chunk_channels as f32).unwrap();
                            }
                        }
                    }
                    2 => {
                        if chunk_channels == 1 {
                            for (i, sample) in chunk.samples(0).iter().enumerate() {
                                mixed_chunk.samples_mut(0)[i] =
                                    *sample * S::from_f32(volume).unwrap();
                                mixed_chunk.samples_mut(1)[i] =
                                    *sample * S::from_f32(volume).unwrap();
                            }
                        } else if chunk_channels == 2 {
                            for c in 0..chunk_channels {
                                for (i, sample) in chunk.samples(c).iter().enumerate() {
                                    mixed_chunk.samples_mut(c)[i] =
                                        *sample * S::from_f32(volume).unwrap();
                                }
                            }
                        } else {
                            unimplemented!()
                        }
                    }
                    _ => unimplemented!(),
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
