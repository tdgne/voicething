use super::super::common::*;
use super::node::*;
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

#[derive(Getters, Serialize, Deserialize, Debug)]
pub struct Windower<S: Sample> {
    #[serde(skip)]
    input: Option<ChunkReceiver<S>>,
    #[serde(skip)]
    outputs: Vec<SyncChunkSender<S>>,
    #[serde(skip)]
    id: Uuid,
    #[serde(skip)]
    buffer: Vec<VecDeque<S>>,
    window_function: WindowFunction,
    #[getset(get = "pub")]
    window_size: usize,
    #[getset(get = "pub")]
    delay: usize,
}

impl<S: Sample> HasId for Windower<S> {
    fn id(&self) -> Uuid {
        self.id
    }
}

impl<S: Sample> Windower<S> {
    pub fn new(window_function: WindowFunction, window_size: usize, delay: usize) -> Self {
        Self {
            input: None,
            outputs: vec![],
            id: Uuid::new_v4(),
            window_function,
            window_size,
            delay,
            buffer: vec![],
        }
    }

    pub fn window_function_mut(&mut self) -> &mut WindowFunction {
        &mut self.window_function
    }

    pub fn window_size_mut(&mut self) -> &mut usize {
        &mut self.window_size
    }

    pub fn delay_mut(&mut self) -> &mut usize {
        &mut self.delay
    }

    pub fn input(&self) -> Option<&ChunkReceiver<S>> {
        self.input.as_ref()
    }

    pub fn outputs(&self) -> &[SyncChunkSender<S>] {
        self.outputs.as_ref()
    }

    pub fn set_input(&mut self, rx: Option<ChunkReceiver<S>>) {
        self.input = rx;
    }

    pub fn add_output(&mut self, tx: SyncChunkSender<S>) {
        self.outputs.push(tx);
    }

    fn triangular_window(x: usize, length: usize) -> f32 {
        let x = x as f32 / length as f32;
        1.0 - (x - 0.5).abs() * 2.0
    }

    fn hanning_window(x: usize, length: usize) -> f32 {
        let x = x as f32 / length as f32;
        0.5 - 0.5 * (2.0 * 3.141592 * x).cos()
    }

    pub fn process_chunk(&mut self, chunk: SampleChunk<S>) -> Vec<SampleChunk<S>> {
        for c in 0..*chunk.metadata().channels() {
            if self.buffer.len() <= c {
                self.buffer.push(vec![].into());
            }
            for s in chunk.samples(c).iter() {
                self.buffer[c].push_back(*s);
            }
        }
        let mut windowed_chunks = vec![];
        while self.buffer[0].len() >= self.window_size {
            let mut windowed_chunk: SampleChunk<S> = SampleChunk::from_flat_samples(
                &vec![S::from_f32(0.0).unwrap(); self.buffer.len() * self.window_size],
                chunk.metadata().clone(),
            )
            .unwrap();
            windowed_chunk.set_window_info(Some(WindowInfo::new(self.window_function.clone(), self.delay)));
            for (c, b) in self.buffer.iter().enumerate() {
                let samples = windowed_chunk.samples_mut(c);
                for (i, s) in b.iter().take(self.window_size).enumerate() {
                    samples[i] = *s * match self.window_function {
                        WindowFunction::Rectangular => S::from_f32(1.0f32).unwrap(),
                        WindowFunction::Hanning => S::from_f32(Self::hanning_window(i, self.window_size)).unwrap(),
                        WindowFunction::Triangular => S::from_f32(Self::triangular_window(i, self.window_size)).unwrap(),
                    };
                }
            }
            for b in self.buffer.iter_mut() {
                for _ in 0..self.delay {
                    b.pop_front();
                }
            }
            windowed_chunks.push(windowed_chunk);
        }
        windowed_chunks
    }

    pub fn run_once(&mut self) {
        if let Some(input) = self.input() {
            if let Some(chunk) = input.try_recv().ok() {
                let chunks = self.process_chunk(chunk);
                for output in self.outputs().iter() {
                    for chunk in chunks.iter() {
                        let _ = output.try_send(chunk.clone());
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::audio::*;
    #[test]
    fn window_dewindow() {
        let c = SampleChunk::from_flat_samples(&vec![1.0; 1024*2], AudioMetadata::new(2, 44100)).unwrap();
        let mut w = Windower::new(WindowFunction::Hanning, 1024, 128);
        let mut dw = Dewindower::new(1024);
        let mut first = true;
        for _ in 0..4 {
            for n in w.process_chunk(c.clone()).into_iter() {
                assert_eq!(*n.duration_samples(), 1024);
                assert_ne!(n.samples(0)[1], 0.0);
                for n in dw.process_chunk(n).iter() {
                    assert_eq!(*n.duration_samples(), 1024);
                    if !first {
                        for s in n.samples(0) {
                            assert_ne!(*s, 0.0);
                        }
                    } else {
                        assert_eq!(n.samples(0)[0], 0.0);
                    }
                    first = false;
                }
            }
        }
    }

    #[test]
    fn window_dewindow_changing_params() {
        let c = SampleChunk::from_flat_samples(&vec![1.0; 1024*2], AudioMetadata::new(2, 44100)).unwrap();
        let mut w = Windower::new(WindowFunction::Hanning, 1024, 128);
        let mut dw = Dewindower::new(1024);

        for i in 0..6 {
            if i == 2 {
                w = Windower::new(WindowFunction::Hanning, 300, 50);
            }
            if i == 4 {
                w = Windower::new(WindowFunction::Hanning, 1024, 64);
            }
            for n in w.process_chunk(c.clone()).into_iter() {
                dw.process_chunk(n);
            }
        }
    }
}
