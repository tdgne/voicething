use crate::audio::common::{SampleChunk};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, SyncSender};
use uuid::Uuid;

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

#[derive(Serialize, Deserialize, Debug)]
pub struct InputPort {
    #[serde(skip)]
    pub rx: Option<Receiver<SampleChunk>>,
    id: Uuid,
    node_id: Uuid,
    pub output_id: Option<Uuid>,
}

impl InputPort {
    pub fn new(node_id: Uuid) -> Self {
        Self {
            rx: None,
            id: Uuid::new_v4(),
            node_id,
            output_id: None,
        }
    }

    pub fn id(&self) -> Uuid {
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
    id: Uuid,
    node_id: Uuid,
    pub input_id: Option<Uuid>,
}

impl OutputPort {
    pub fn new(node_id: Uuid) -> Self {
        Self {
            tx: None,
            id: Uuid::new_v4(),
            node_id,
            input_id: None,
        }
    }

    pub fn id(&self) -> Uuid {
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


