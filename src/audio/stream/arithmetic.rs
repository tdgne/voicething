use super::super::common::*;
use super::node::*;
use getset::Getters;
use rustfft::num_complex::Complex32;
use rustfft::num_traits::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
pub enum ArithmeticOperation {
    Multiply(f32),
    Log,
    Exp,
    Reciprocal,
    Inverse,
    Abs,
}

#[derive(Getters, Serialize, Deserialize, Debug)]
pub struct ArithmeticNode {
    io: NodeIo,
    id: NodeId,
    op: ArithmeticOperation,
}

impl HasNodeIo for ArithmeticNode {
    fn node_io(&self) -> &NodeIo {
        &self.io
    }
    fn node_io_mut(&mut self) -> &mut NodeIo {
        &mut self.io
    }
}

impl ArithmeticNode {
    pub fn new(op: ArithmeticOperation) -> Self {
        Self {
            io: NodeIo::new(),
            id: NodeId::new(),
            op,
        }
    }

    pub fn op(&self) -> &ArithmeticOperation {
        &self.op
    }
    pub fn op_mut(&mut self) -> &mut ArithmeticOperation {
        &mut self.op
    }

    pub fn process_chunk(&self, chunk: DataChunk) -> DataChunk {
        let samples = match &chunk {
            DataChunk::Real(chunk) => chunk
                .flattened_data()
                .iter()
                .map(|s| Complex32::from_f32(*s).unwrap())
                .collect::<Vec<_>>(),
            DataChunk::Complex(chunk) => chunk.flattened_data(),
        }
        .iter()
        .map(|s| match self.op {
            ArithmeticOperation::Multiply(c) => s * c,
            ArithmeticOperation::Log => s.ln(),
            ArithmeticOperation::Exp => s.exp(),
            ArithmeticOperation::Reciprocal => s.finv(),
            ArithmeticOperation::Inverse => -s,
            ArithmeticOperation::Abs => Complex32::from_f32(s.norm()).unwrap(),
        })
        .collect::<Vec<_>>();
        match &chunk {
            DataChunk::Real(_) => {
                let mut new_chunk = GenericDataChunk::from_flat_sata(
                    &samples.iter().map(|s| s.re).collect::<Vec<_>>(),
                    chunk.metadata().clone(),
                )
                .unwrap();
                new_chunk.set_window_info(chunk.window_info().clone());
                DataChunk::Real(new_chunk)
            }
            DataChunk::Complex(_) => {
                let mut new_chunk =
                    GenericDataChunk::from_flat_sata(&samples, chunk.metadata().clone())
                        .unwrap();
                new_chunk.set_window_info(chunk.window_info().clone());
                DataChunk::Complex(new_chunk)
            }
        }
    }
}

impl NodeTrait for ArithmeticNode {
    fn id(&self) -> NodeId {
        self.id
    }
    fn run_once(&mut self) {
        if self.inputs().len() != 1 {
            return;
        }
        while let Some(chunk) = self.inputs()[0].try_recv().ok() {
            let chunk = self.process_chunk(chunk);
            for output in self.outputs().iter() {
                let result = output.try_send(chunk.clone());
            }
        }
    }
}
