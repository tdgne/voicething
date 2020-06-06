use super::super::common::*;
use super::node::*;
use super::port::*;
use getset::Getters;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Getters, Serialize, Deserialize, Debug)]
pub struct IdentityNode {
    inputs: Vec<InputPort>,
    outputs: Vec<OutputPort>,
    name: String,
    id: Uuid,
}

impl IdentityNode {
    pub fn new(name: String) -> Self {
        Self {
            inputs: vec![],
            outputs: vec![],
            name,
            id: Uuid::new_v4(),
        }
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl NodeTrait for IdentityNode {
    fn id(&self) -> Uuid {
        self.id
    }
    fn inputs(&self) -> &[InputPort] {
        &self.inputs
    }
    fn outputs(&self) -> &[OutputPort] {
        &self.outputs
    }
    fn inputs_mut(&mut self) -> &mut [InputPort] {
        &mut self.inputs
    }
    fn outputs_mut(&mut self) -> &mut [OutputPort] {
        &mut self.outputs
    }
    fn add_input(&mut self) -> Result<&mut InputPort, Box<dyn std::error::Error>> {
        if self.inputs.len() == 0 {
            self.inputs.push(InputPort::new(self.id));
            Ok(&mut self.inputs[0])
        } else {
            Err(Box::new(PortAdditionError))
        }
    }
    fn add_output(&mut self) -> Result<&mut OutputPort, Box<dyn std::error::Error>> {
        self.outputs.push(OutputPort::new(self.id));
        let l = self.outputs.len();
        Ok(&mut self.outputs[l - 1])
    }
    fn run_once(&mut self) {
        if self.inputs.len() != 1 {
            return;
        }
        if let Some(chunk) = self.inputs[0].try_recv().ok() {
            for output in self.outputs().iter() {
                let result = output.try_send(chunk.clone());
            }
        }
    }
}
