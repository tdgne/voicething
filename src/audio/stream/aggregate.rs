use super::super::common::*;
use super::node::*;
use serde::{Deserialize, Serialize};
use rustfft::num_complex::Complex32;
use rustfft::num_traits::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct AggregateNode {
    io: NodeIo,
    id: NodeId,
    setting: AggregateSetting,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AggregateSetting {
    Sum,
    Product,
}

impl HasNodeIo for AggregateNode {
    fn node_io(&self) -> &NodeIo {
        &self.io
    }
    fn node_io_mut(&mut self) -> &mut NodeIo {
        &mut self.io
    }
}

impl AggregateNode {
    pub fn new(setting: AggregateSetting) -> Self {
        Self {
            io: NodeIo::new(),
            id: NodeId::new(),
            setting,
        }
    }

    pub fn setting(&self) -> &AggregateSetting {
        &self.setting
    }

    pub fn process_chunk(&mut self, chunks: Vec<SampleChunk>) -> Option<SampleChunk> {
        if chunks.len() != self.inputs().len() || chunks.len() == 0 {
            return None;
        }
        let is_real = chunks
            .iter()
            .map(|c| match c {
                SampleChunk::Real(_) => true,
                _ => false,
            })
            .fold(true, |acc, cur| acc && cur);
        let samples = chunks
            .iter()
            .map(|c| match c {
                SampleChunk::Real(c) => c
                    .flattened_samples()
                    .iter()
                    .map(|s| Complex32::from_f32(*s).unwrap())
                    .collect::<Vec<_>>(),
                SampleChunk::Complex(c) => c.flattened_samples(),
            })
            .collect::<Vec<_>>();
        let l = samples[0].len();
        let new_samples = match &self.setting {
            AggregateSetting::Sum => (0..l)
                .map(|i| {
                    (0..samples.len())
                        .map(|k| samples[k][i])
                        .fold(Complex32::from_f32(0.0).unwrap(), |acc, cur| acc + cur)
                })
                .collect::<Vec<_>>(),
            AggregateSetting::Product => (0..l)
                .map(|i| {
                    (0..samples.len())
                        .map(|k| samples[k][i])
                        .fold(Complex32::from_f32(1.0).unwrap(), |acc, cur| acc * cur)
                })
                .collect::<Vec<_>>(),
        };
        let metadata = chunks[0].metadata().clone();
        if is_real {
            let new_samples = new_samples.iter().map(|s| s.re).collect::<Vec<_>>();
            Some(SampleChunk::Real(
                GenericSampleChunk::from_flat_samples(&new_samples, metadata)
                    .unwrap(),
            ))
        } else {
            Some(SampleChunk::Complex(
                GenericSampleChunk::from_flat_samples(&new_samples, metadata)
                    .unwrap(),
            ))
        }
    }
}

impl NodeTrait for AggregateNode {
    fn id(&self) -> NodeId {
        self.id
    }
    fn run_once(&mut self) {
        let chunks = self
            .inputs()
            .iter()
            .flat_map(|input| input.try_recv().ok())
            .collect::<Vec<_>>();
        let chunk = self.process_chunk(chunks).unwrap();
        for output in self.outputs().iter() {
            let _ = output.try_send(chunk.clone());
        }
    }
    fn add_input(&mut self) -> Result<&mut InputPort, Box<dyn std::error::Error>> {
        let id = self.id();
        self.node_io_mut().inputs_mut().push(InputPort::new(id));
        let l = self.inputs().len();
        Ok(&mut self.inputs_mut()[l - 1])
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sum() {
        let setting = AggregateSetting::Sum;
        let metadata = AudioMetadata::new(2, 44100);
        let c = vec![SampleChunk::Real(GenericSampleChunk::from_flat_samples(&vec![1.0; 2048], metadata).unwrap()); 2];
        let mut n = AggregateNode::new(setting);
        n.add_input();
        n.add_input();
        let c = n.process_chunk(c).unwrap();
        match c {
            SampleChunk::Real(c) => assert_eq!(c.samples(0)[0], 2.0),
            _ => panic!(),
        }
    }

    #[test]
    fn product() {
        let setting = AggregateSetting::Product;
        let metadata = AudioMetadata::new(2, 44100);
        let c = vec![SampleChunk::Real(GenericSampleChunk::from_flat_samples(&vec![2.0; 2048], metadata).unwrap()); 2];
        let mut n = AggregateNode::new(setting);
        n.add_input();
        n.add_input();
        let c = n.process_chunk(c).unwrap();
        match c {
            SampleChunk::Real(c) => assert_eq!(c.samples(1)[1023], 4.0),
            _ => panic!(),
        }
    }
}
