use super::super::common::*;
use super::node::*;
use getset::Getters;
use serde::{Deserialize, Serialize};

#[derive(Getters, Serialize, Deserialize, Debug)]
pub struct IdentityNode {
    io: NodeIo,
    name: String,
    id: NodeId,
    #[serde(skip)]
    last_chunk: Option<DataChunk>,
}

impl HasNodeIo for IdentityNode {
    fn node_io(&self) -> &NodeIo {
        &self.io
    }
    fn node_io_mut(&mut self) -> &mut NodeIo {
        &mut self.io
    }
}

impl IdentityNode {
    pub fn new(name: String) -> Self {
        Self {
            io: NodeIo::new(),
            name,
            id: NodeId::new(),
            last_chunk: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn last_chunk(&self) -> Option<&DataChunk> {
        self.last_chunk.as_ref()
    }
}

impl NodeTrait for IdentityNode {
    fn id(&self) -> NodeId {
        self.id
    }
    fn run_once(&mut self) {
        if self.inputs().len() != 1 {
            return;
        }
        while let Some(chunk) = self.inputs()[0].try_recv().ok() {
            self.last_chunk = Some(chunk.clone());
            for output in self.outputs().iter() {
                let result = output.try_send(chunk.clone());
            }
        }
    }
}
