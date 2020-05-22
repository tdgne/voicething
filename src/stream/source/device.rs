use crate::common::*;
use getset::Getters;
use cpal::traits::{DeviceTrait, HostTrait, EventLoopTrait};
use std::sync::mpsc::{channel, Receiver, Sender};
use crate::stream::{Node, SingleOutputNode};
use std::collections::VecDeque;

#[derive(Getters)]
pub struct RecordingSource {
    sender: Option<Sender<SampleChunk<f32>>>,
    buffer: VecDeque<f32>,
    chunk_duration: usize,
}

impl RecordingSource {
    pub fn new(chunk_duration: usize) -> Self {
        Self {
            sender: None,
            buffer: VecDeque::new(),
            chunk_duration,
        }
    }
}

impl Node for RecordingSource {
    fn run(&mut self) {
        let host = cpal::default_host();
        let device = host.default_input_device().unwrap();
        let format = device.default_input_format().unwrap();
        let event_loop = host.event_loop();
        let stream_id = event_loop.build_input_stream(&device, &format).unwrap();
        let metadata = AudioMetadata::new(
            format.channels as usize,
            format.sample_rate.0 as usize,
            );
        event_loop.play_stream(stream_id).unwrap();
        event_loop.run(move |_stream_id, stream_result| {
            let stream_data = match stream_result {
                Ok(stream_data) => stream_data,
                _ => return,
            };

            let input_buffer = match stream_data {
                cpal::StreamData::Input{buffer} => buffer,
                _ => return,
            };

            let sender = self.sender.as_mut().unwrap();

            match input_buffer {
                cpal::UnknownTypeInputBuffer::F32(input_buffer) => {
                    for sample in input_buffer.iter() {
                        self.buffer.push_back(*sample);
                    }
                    let channels = metadata.channels();
                    let chunk_duration = self.chunk_duration;
                    if self.buffer.len() >= channels * chunk_duration {
                        let mut samples = vec![];
                        for i in 0..channels * chunk_duration {
                            samples.push(self.buffer.pop_front().unwrap());
                        }

                        let chunk = SampleChunk::from_flat_samples(
                            &samples,
                            metadata.clone(),
                            )
                            .unwrap();

                        sender.send(chunk).unwrap();
                    }
                }
                _ => unimplemented!(),
            }
        });
    }
}

impl SingleOutputNode<f32> for RecordingSource {
    fn output(&mut self) -> Receiver<SampleChunk<f32>> {
        let (sender, receiver) = channel();
        self.sender = Some(sender);
        receiver
    }
}
