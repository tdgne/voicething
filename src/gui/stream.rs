pub mod aggregate;
pub mod arithmetic;
pub mod dewindower;
pub mod ft;
pub mod filter;
pub mod identity;
pub mod node;
pub mod port;
pub mod psola;
pub mod windower;
pub mod phasevocoder;
pub use aggregate::*;
pub use arithmetic::*;
pub use dewindower::*;
pub use ft::*;
pub use filter::*;
pub use identity::*;
pub use node::*;
pub use port::*;
pub use psola::*;
pub use windower::*;
pub use phasevocoder::*;

use std::collections::HashMap;
use crate::audio::stream::node::*;
use crate::audio::stream::graph::Graph;
use imgui::*;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};

pub type ConnectRequest = (OutputPortId, InputPortId);

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeEditorState {
    graph: Arc<Mutex<Graph>>,
    node_pos: HashMap<NodeId, [f32; 2]>,
    input_pos: HashMap<InputPortId, [f32; 2]>,
    output_pos: HashMap<OutputPortId, [f32; 2]>,
    left_dragged: Option<NodeId>,
    right_dragged: Option<OutputPortId>,
    window_opened: HashMap<NodeId, bool>,
}

impl NodeEditorState {
    pub fn new(graph: Arc<Mutex<Graph>>) -> Self {
        Self {
            graph,
            node_pos: HashMap::new(),
            input_pos: HashMap::new(),
            output_pos: HashMap::new(),
            left_dragged: None,
            right_dragged: None,
            window_opened: HashMap::new(),
        }
    }

    pub fn graph(&self) -> Arc<Mutex<Graph>> {
        self.graph.clone()
    }

    pub fn set_node_pos(&mut self, uuid: NodeId, pos: [f32; 2]) {
        self.node_pos.insert(uuid, pos);
    }

    pub fn set_input_pos(&mut self, uuid: InputPortId, pos: [f32; 2]) {
        self.input_pos.insert(uuid, pos);
    }

    pub fn set_output_pos(&mut self, uuid: OutputPortId, pos: [f32; 2]) {
        self.output_pos.insert(uuid, pos);
    }

    pub fn node_pos(&self, uuid: &NodeId) -> Option<&[f32; 2]> {
        self.node_pos.get(uuid)
    }

    pub fn input_pos(&self, uuid: &InputPortId) -> Option<&[f32; 2]> {
        self.input_pos.get(uuid)
    }

    pub fn output_pos(&self, uuid: &OutputPortId) -> Option<&[f32; 2]> {
        self.output_pos.get(uuid)
    }

    pub fn node_pos_mut(&mut self, uuid: &NodeId) -> Option<&mut [f32; 2]> {
        self.node_pos.get_mut(uuid)
    }

    pub fn set_left_dragged(&mut self, uuid: Option<NodeId>) {
        self.left_dragged = uuid;
    }

    pub fn left_dragged(&self) -> Option<NodeId> {
        self.left_dragged
    }

    pub fn set_right_dragged(&mut self, uuid: Option<OutputPortId>) {
        self.right_dragged = uuid;
    }

    pub fn right_dragged(&self) -> Option<OutputPortId> {
        self.right_dragged
    }

    pub fn window_opened(&mut self, id: &NodeId) -> &bool {
        if self.window_opened.get(id).is_none() {
            self.window_opened.insert(id.clone(), false);
        }
        self.window_opened.get(id).unwrap()
    }

    pub fn window_opened_mut(&mut self, id: &NodeId) -> &mut bool {
        if self.window_opened.get(id).is_none() {
            self.window_opened.insert(id.clone(), false);
        }
        self.window_opened.get_mut(id).unwrap()
    }
}


