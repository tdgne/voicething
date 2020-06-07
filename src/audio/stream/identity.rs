use super::super::common::*;
use super::node::*;
use getset::Getters;
use serde::{Serialize, Deserialize};

#[derive(Getters, Serialize, Deserialize, Debug)]
pub struct IdentityNode {
    io: NodeIo,
    name: String,
    id: NodeId,
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
        }
    }
    
    pub fn name(&self) -> &str {
        &self.name
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
        if let Some(chunk) = self.inputs()[0].try_recv().ok() {
            for output in self.outputs().iter() {
                let result = output.try_send(chunk.clone());
            }
        }
    }
}
