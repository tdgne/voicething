use crate::audio::common::{SampleChunk};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, SyncSender};
use uuid::Uuid;
#[macro_use]
use enum_dispatch::enum_dispatch;

use super::*;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct NodeId(Uuid);

impl NodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeIo {
    inputs: Vec<InputPort>,
    outputs: Vec<OutputPort>,
}

impl NodeIo {
    pub fn new() -> Self {
        Self {
            inputs: vec![],
            outputs: vec![],
        }
    }
    pub fn inputs(&self) -> &[InputPort] {
        &self.inputs
    }
    pub fn outputs(&self) -> &[OutputPort] {
        &self.outputs
    }
    pub fn inputs_mut(&mut self) -> &mut Vec<InputPort> {
        &mut self.inputs
    }
    pub fn outputs_mut(&mut self) -> &mut Vec<OutputPort> {
        &mut self.outputs
    }
}

#[enum_dispatch]
pub trait HasNodeIo {
    fn node_io(&self) -> &NodeIo;
    fn node_io_mut(&mut self) -> &mut NodeIo;
}

#[enum_dispatch]
pub trait NodeTrait: HasNodeIo {
    fn id(&self) -> NodeId;
    fn inputs(&self) -> &[InputPort] {
        self.node_io().inputs()
    }
    fn outputs(&self) -> &[OutputPort] {
        self.node_io().outputs()
    }
    fn inputs_mut(&mut self) -> &mut [InputPort] {
        self.node_io_mut().inputs_mut()
    }
    fn outputs_mut(&mut self) -> &mut [OutputPort] {
        self.node_io_mut().outputs_mut()
    }
    fn add_input(&mut self) -> Result<&mut InputPort, Box<dyn std::error::Error>> {
        if self.inputs().len() == 0 {
            let id = self.id();
            self.node_io_mut().inputs_mut().push(InputPort::new(id));
            Ok(&mut self.inputs_mut()[0])
        } else {
            Err(Box::new(PortAdditionError))
        }
    }
    fn add_output(&mut self) -> Result<&mut OutputPort, Box<dyn std::error::Error>> {
        let id = self.id();
        self.node_io_mut().outputs_mut().push(OutputPort::new(id));
        let l = self.outputs().len();
        Ok(&mut self.outputs_mut()[l - 1])
    }
    fn run_once(&mut self);
}

#[enum_dispatch(NodeTrait)]
#[enum_dispatch(HasNodeIo)]
#[derive(Serialize, Deserialize, Debug)]
pub enum Node {
    Psola(PsolaNode),
    Windower(Windower),
    Dewindower(Dewindower),
    Identity(IdentityNode),
    Aggregate(AggregateNode),
    FourierTransform(FourierTransform),
    Arithmetic(ArithmeticNode),
}

#[derive(Debug, Clone)]
pub struct PortAdditionError;

impl std::fmt::Display for PortAdditionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "could not add an input or output port")
    }
}

impl std::error::Error for PortAdditionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[derive(Debug, Clone)]
struct NoSenderReceiverError;

impl std::fmt::Display for NoSenderReceiverError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "no sender or receiver")
    }
}

impl std::error::Error for NoSenderReceiverError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct InputPortId(Uuid);

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct OutputPortId(Uuid);

impl InputPortId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl OutputPortId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InputPort {
    #[serde(skip)]
    pub rx: Option<Receiver<SampleChunk>>,
    id: InputPortId,
    node_id: NodeId,
    pub output_id: Option<OutputPortId>,
}

impl InputPort {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            rx: None,
            id: InputPortId::new(),
            node_id,
            output_id: None,
        }
    }

    pub fn id(&self) -> InputPortId {
        self.id
    }

    pub fn try_recv(&self) -> Result<SampleChunk, Box::<dyn std::error::Error>> {
        match &self.rx {
            Some(rx) => {
                Ok(rx.try_recv()?)
            },
            None => Err(NoSenderReceiverError)?
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OutputPort {
    #[serde(skip)]
    pub tx: Option<SyncSender<SampleChunk>>,
    id: OutputPortId,
    node_id: NodeId,
    pub input_id: Option<InputPortId>,
}

impl OutputPort {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            tx: None,
            id: OutputPortId::new(),
            node_id,
            input_id: None,
        }
    }

    pub fn id(&self) -> OutputPortId {
        self.id
    }

    pub fn try_send(&self, chunk: SampleChunk) -> Result<(), Box::<dyn std::error::Error>> {
        match &self.tx {
            Some(tx) => {
                Ok(tx.try_send(chunk)?)
            },
            None => Err(NoSenderReceiverError)?
        }
    }
}


