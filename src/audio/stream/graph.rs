use super::node::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::{
    fmt,
    fmt::{Display, Formatter},
};
use std::sync::mpsc::{sync_channel};

#[derive(Serialize, Deserialize, Debug)]
pub struct Graph {
    nodes: HashMap<NodeId, Arc<Mutex<Node>>>,
    edges: HashMap<OutputPortId, InputPortId>,
    input_port_node_map: HashMap<InputPortId, NodeId>,
    output_port_node_map: HashMap<OutputPortId, NodeId>,
}

#[derive(Debug, Clone)]
struct CompatibilityError;

impl Display for CompatibilityError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "incompatible kinds of nodes")
    }
}

impl Error for CompatibilityError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Debug, Clone)]
struct ExistenceError(&'static str);

impl Display for ExistenceError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "the {} doesn't exist", self.0)
    }
}

impl Error for ExistenceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            input_port_node_map: HashMap::new(),
            output_port_node_map: HashMap::new(),
        }
    }

    pub fn is_input_port(&self, id: &InputPortId) -> bool {
        self.input_port_node_map.get(id).is_some()
    }

    pub fn is_output_port(&self, id: &OutputPortId) -> bool {
        self.output_port_node_map.get(id).is_some()
    }

    pub fn add_input(&mut self, node_id: &NodeId) -> Result<InputPortId, Box<dyn Error>> {
        let node = self.node(node_id)?;
        let id = node.lock().unwrap().add_input().unwrap().id().clone();
        self.input_port_node_map.insert(id, *node_id);
        Ok(id)
    }

    pub fn add_output(&mut self, node_id: &NodeId) -> Result<OutputPortId, Box<dyn Error>> {
        let node = self.node(node_id)?;
        let id = node.lock().unwrap().add_output().unwrap().id().clone();
        self.output_port_node_map.insert(id, *node_id);
        Ok(id)
    }

    fn bfs_run_once(&self, node: Arc<Mutex<Node>>) -> Result<(), Box<dyn Error>> {
        let mut q = VecDeque::new();
        q.push_back(node);
        while !q.is_empty() {
            let node = q.pop_front().unwrap();
            node.lock().unwrap().run_once();
            for output_port in node.lock().unwrap().outputs() {
                if let Some(other_end_port_id) = self
                    .edges
                    .get(&output_port.id())
                {
                    let other_end_node_id = self.input_port_node_map.get(&other_end_port_id).unwrap();
                    q.push_back(self.node(&other_end_node_id)?);
                }
            }
        }
        Ok(())
    }

    pub fn run_once(&self) -> Result<(), Box<dyn Error>> {
        let input = {
            let mut input = Err(ExistenceError("Input"));
            for (_, v) in self.nodes.iter() {
                if let Node::Identity(node) = &*v.lock().unwrap() {
                    if node.name() == "Input" {
                        input = Ok(v.clone());
                    }
                }
            }
            input?
        };
        // TODO: think of a wiser way
        for _ in 0..self.nodes.len() {
            self.bfs_run_once(input.clone())?;
        }
        Ok(())
    }

    pub fn nodes(&self) -> &HashMap<NodeId, Arc<Mutex<Node>>> {
        &self.nodes
    }

    pub fn edges(&self) -> &HashMap<OutputPortId, InputPortId> {
        &self.edges
    }

    pub fn add(&mut self, node: Node) {
        self.nodes.insert(node.id(), Arc::new(Mutex::new(node)));
    }

    pub fn remove(&mut self, id: NodeId) -> Option<Arc<Mutex<Node>>> {
        let node = self.node(&id).unwrap();
        let input_ids = node.lock().unwrap().inputs().iter().map(|p| p.id()).collect::<Vec<_>>();
        let output_ids = node.lock().unwrap().outputs().iter().map(|p| p.id()).collect::<Vec<_>>();
        for port in input_ids.iter() {
            self.detach_input_port(port);
        }
        for port in output_ids.iter() {
            self.detach_output_port(port);
        }
        self.nodes.remove(&id)
    }

    pub fn node(&self, id: &NodeId) -> Result<Arc<Mutex<Node>>, ExistenceError> {
        if let Some(ref node) = self.nodes.get(&id) {
            Ok(Arc::clone(node))
        } else {
            Err(ExistenceError("node"))
        }
    }

    pub fn connect_ports(&mut self, from_id: &OutputPortId, to_id: &InputPortId) -> Result<(), Box<dyn Error>> {
        self.detach_output_port(from_id);
        self.detach_input_port(to_id);
        let from_node = self.node(self.output_port_node_map.get(from_id).unwrap())?;
        let to_node = self.node(self.input_port_node_map.get(to_id).unwrap())?;
        let (tx, rx) = sync_channel(16);
        for port in from_node.lock().unwrap().outputs_mut().iter_mut() {
            if port.id() == *from_id {
                port.tx = Some(tx);
                port.input_id = Some(*to_id);
                break;
            }
        }
        for port in to_node.lock().unwrap().inputs_mut().iter_mut() {
            if port.id() == *to_id {
                port.rx = Some(rx);
                port.output_id = Some(*from_id);
                break;
            }
        }
        self.edges.insert(*from_id, *to_id);
        Ok(())
    }

    pub fn disconnect_ports(&mut self, from_id: &OutputPortId, to_id: &InputPortId) -> Result<(), Box<dyn Error>> {
        let from_node = self.node(self.output_port_node_map.get(from_id).unwrap())?;
        let to_node = self.node(self.input_port_node_map.get(to_id).unwrap())?;
        for port in from_node.lock().unwrap().outputs_mut().iter_mut() {
            if port.id() == *from_id {
                port.tx = None;
                port.input_id = None;
                break;
            }
        }
        for port in to_node.lock().unwrap().inputs_mut().iter_mut() {
            if port.id() == *to_id {
                port.rx = None;
                port.output_id = None;
                break;
            }
        }
        self.edges.remove(&from_id);
        Ok(())
    }

    pub fn detach_output_port(&mut self, id: &OutputPortId) -> Result<(), Box<dyn Error>> {
        let mut other_id = None;
        for (k, v) in self.edges.iter() {
            if *k == *id {
                other_id = Some(*v);
            }
        }
        if let Some(other_id) = other_id {
            if self.output_port_node_map.get(id).is_some() {
                self.disconnect_ports(id, &other_id)
            } else {
                panic!("bad graph data");
            }
        } else {
            Ok(())
        }
    }

    pub fn detach_input_port(&mut self, id: &InputPortId) -> Result<(), Box<dyn Error>> {
        let mut other_id = None;
        for (k, v) in self.edges.iter() {
            if *v == *id {
                other_id = Some(*k);
            }
        }
        if let Some(other_id) = other_id {
            if self.input_port_node_map.get(id).is_some() {
                self.disconnect_ports(&other_id, id)
            } else {
                panic!("bad graph data");
            }
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::*;
    #[test]
    fn connect_run_disconnect() {
        let mut g = Graph::new();
        let n1 = Node::Identity(IdentityNode::new("Input".to_string()));
        let n2 = Node::Psola(PsolaNode::new(1.0));
        let n1_id = n1.id();
        let n2_id = n2.id();
        g.add(n1);
        g.add(n2);
        let n1_out_id = g.add_output(&n1_id).unwrap();
        let n2_in_id = g.add_input(&n2_id).unwrap();
        g.connect_ports(&n1_out_id, &n2_in_id).unwrap();
        assert_eq!(*g.edges().get(&n1_out_id).unwrap(), n2_in_id);
        assert_eq!(*g.input_port_node_map.get(&n2_in_id).unwrap(), n2_id);
        match &*g.node(&n1_id).unwrap().lock().unwrap() {
            Node::Identity(ref n) => assert_eq!(n.outputs().len(), 1),
            _ => panic!(),
        };
        match &*g.node(&n2_id).unwrap().lock().unwrap() {
            Node::Psola(ref n) => assert_eq!(n.inputs().len(), 1),
            _ => panic!(),
        };
        g.run_once().unwrap();
        g.disconnect_ports(&n1_out_id, &n2_in_id).unwrap();
        assert_eq!(g.edges().get(&n1_out_id), None);
    }
}
