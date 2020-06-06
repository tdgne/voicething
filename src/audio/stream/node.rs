use crate::audio::common::{SampleChunk};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, SyncSender};
use uuid::Uuid;
#[macro_use]
use enum_dispatch::enum_dispatch;

use super::*;

#[enum_dispatch]
pub trait NodeTrait {
    fn id(&self) -> Uuid;
    fn inputs(&self) -> &[InputPort];
    fn outputs(&self) -> &[OutputPort];
    fn inputs_mut(&mut self) -> &mut [InputPort];
    fn outputs_mut(&mut self) -> &mut [OutputPort];
    fn add_input(&mut self) -> Result<&mut InputPort, Box<dyn std::error::Error>>;
    fn add_output(&mut self) -> Result<&mut OutputPort, Box<dyn std::error::Error>>;
    fn run_once(&mut self);
}

#[enum_dispatch(NodeTrait)]
#[derive(Serialize, Deserialize, Debug)]
pub enum Node {
    Psola(PsolaNode),
    Identity(IdentityNode),
}
