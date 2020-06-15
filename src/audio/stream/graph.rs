use super::identity::*;
use super::node::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use std::sync::mpsc::sync_channel;
use std::sync::{Arc, Mutex};
use std::{
    fmt,
    fmt::{Display, Formatter},
};

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

    pub fn default() -> Self {
        let mut g = Self::new();

        let input_node = IdentityNode::new("Input".to_string());
        let input_node_id = input_node.id();
        g.add(Node::Identity(input_node));

        let output_node = IdentityNode::new("Output".to_string());
        let output_node_id = output_node.id();
        g.add(Node::Identity(output_node));

        {
            let p1 = g.add_output(&input_node_id).unwrap();
            let p2 = g.add_input(&output_node_id).unwrap();
            g.connect_ports(&p1, &p2).unwrap();
        }

        g
    }

    pub fn remove_unregistered_ports(&self) {
        for node in self.nodes.values() {
            let mut node = node.lock().unwrap();
            let mut vec = node.outputs_mut();
            loop {
                let remove_index = vec
                    .iter()
                    .enumerate()
                    .filter(|(i, p)| !self.is_output_port(&p.id()))
                    .map(|(i, _)| i)
                    .take(1)
                    .collect::<Vec<_>>();
                if remove_index.len() > 0 {
                    let i = remove_index[0];
                    vec.remove(i);
                } else {
                    break;
                }
            }
            let mut vec = node.inputs_mut();
            loop {
                let remove_index = vec
                    .iter()
                    .enumerate()
                    .filter(|(i, p)| !self.is_input_port(&p.id()))
                    .map(|(i, _)| i)
                    .take(1)
                    .collect::<Vec<_>>();
                if remove_index.len() > 0 {
                    let i = remove_index[0];
                    vec.remove(i);
                } else {
                    break;
                }
            }
        }
    }

    pub fn connect_port_channels(&self) {
        for (oid, iid) in self.edges.iter() {
            let onode_id = self.output_port_node_map.get(&oid).unwrap();
            let inode_id = self.input_port_node_map.get(&iid).unwrap();
            let onode = self.node(onode_id).unwrap();
            let mut onode = onode.lock().unwrap();
            let inode = self.node(inode_id).unwrap();
            let mut inode = inode.lock().unwrap();
            let mut oport = onode
                .outputs_mut()
                .iter_mut()
                .find(|p| p.id() == *oid)
                .unwrap();
            let mut iport = inode
                .inputs_mut()
                .iter_mut()
                .find(|p| p.id() == *iid)
                .unwrap();
            if oport.tx.is_some() && iport.rx.is_some() {
                continue;
            }
            let (tx, rx) = sync_channel(32);
            oport.tx = Some(tx);
            iport.rx = Some(rx);
        }
    }

    fn search_identity_node_by_name(
        &self,
        name: &'static str,
    ) -> Result<Arc<Mutex<Node>>, Box<dyn Error>> {
        let mut input = Err(ExistenceError(name));
        for (_, v) in self.nodes.iter() {
            if let Node::Identity(node) = &*v.lock().unwrap() {
                if node.name() == name.to_string() {
                    input = Ok(v.clone());
                }
            }
        }
        Ok(input?)
    }

    pub fn input_node(&self) -> Result<Arc<Mutex<Node>>, Box<dyn Error>> {
        self.search_identity_node_by_name("Input")
    }

    pub fn output_node(&self) -> Result<Arc<Mutex<Node>>, Box<dyn Error>> {
        self.search_identity_node_by_name("Output")
    }

    pub fn is_input_port(&self, id: &InputPortId) -> bool {
        self.input_port_node_map.get(id).is_some()
    }

    pub fn is_output_port(&self, id: &OutputPortId) -> bool {
        self.output_port_node_map.get(id).is_some()
    }

    pub fn add_input(&mut self, node_id: &NodeId) -> Result<InputPortId, Box<dyn Error>> {
        let node = self.node(node_id)?;
        let id = node.lock().unwrap().add_input()?.id().clone();
        self.input_port_node_map.insert(id, *node_id);
        Ok(id)
    }

    pub fn add_output(&mut self, node_id: &NodeId) -> Result<OutputPortId, Box<dyn Error>> {
        let node = self.node(node_id)?;
        let id = node.lock().unwrap().add_output()?.id().clone();
        self.output_port_node_map.insert(id, *node_id);
        Ok(id)
    }

    fn node_ids_without_inputs(&self) -> Vec<NodeId> {
        let mut s = Vec::new();
        for node in self.nodes.values() {
            let mut has_no_input_edges = node
                .lock()
                .unwrap()
                .inputs()
                .iter()
                .fold(true, |acc, p| acc && !self.edge_with_input_exists(p.id()));
            if has_no_input_edges {
                s.push(node.lock().unwrap().id());
            }
        }
        s
    }

    fn edge_with_input_exists(&self, id: InputPortId) -> bool {
        self.edges.values().find(|v| **v == id).is_some()
    }

    pub fn run_once(&self) -> Result<(), Box<dyn Error>> {
        // topological sort
        let mut s = self.node_ids_without_inputs();
        let mut l = Vec::new();
        let mut removed_edges = HashSet::new();
        while s.len() > 0 {
            let n = s.pop().unwrap();
            l.push(n);
            let n_node = self.node(&n).unwrap();
            let n_node = n_node.lock().unwrap();
            for (e, m) in n_node.outputs().iter().flat_map(|p| {
                p.input_id
                    .map(|id| (id, self.input_port_node_map.get(&id).unwrap()))
            }) {
                removed_edges.insert(e);
                let mut m_has_no_inputs = true;
                let m_node = self.node(m).unwrap();
                let m_node = m_node.lock().unwrap();
                for e in m_node.inputs().iter().map(|p| p.id()) {
                    if !removed_edges.contains(&e) && self.edge_with_input_exists(e) {
                        m_has_no_inputs = false;
                    }
                }
                if m_has_no_inputs {
                    s.push(*m);
                }
            }
        }

        for n in l.iter() {
            self.node(n).unwrap().lock().unwrap().run_once();
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
        let input_ids = node
            .lock()
            .unwrap()
            .inputs()
            .iter()
            .map(|p| p.id())
            .collect::<Vec<_>>();
        let output_ids = node
            .lock()
            .unwrap()
            .outputs()
            .iter()
            .map(|p| p.id())
            .collect::<Vec<_>>();
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

    pub fn connect_ports(
        &mut self,
        from_id: &OutputPortId,
        to_id: &InputPortId,
    ) -> Result<(), Box<dyn Error>> {
        self.detach_output_port(from_id);
        self.detach_input_port(to_id);
        let from_node_id = self.output_port_node_map.get(from_id).unwrap().clone();
        let to_node_id = self.input_port_node_map.get(to_id).unwrap().clone();
        let from_node = self.node(&from_node_id)?;
        let to_node = self.node(&to_node_id)?;
        let (tx, rx) = sync_channel(32);
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
        if self.empty_outputs(&from_node_id) == 0 {
            self.add_output(&from_node_id);
        }
        if self.empty_inputs(&to_node_id) == 0 {
            self.add_input(&to_node_id);
        }
        Ok(())
    }

    pub fn empty_outputs(&self, node_id: &NodeId) -> usize {
        self.node(node_id)
            .unwrap()
            .lock()
            .unwrap()
            .outputs()
            .iter()
            .filter(|p| p.tx.is_none())
            .count()
    }

    pub fn empty_inputs(&self, node_id: &NodeId) -> usize {
        self.node(node_id)
            .unwrap()
            .lock()
            .unwrap()
            .inputs()
            .iter()
            .filter(|p| p.rx.is_none())
            .count()
    }

    pub fn disconnect_ports(
        &mut self,
        from_id: &OutputPortId,
        to_id: &InputPortId,
    ) -> Result<(), Box<dyn Error>> {
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
            Node::Identity(ref n) => assert_eq!(n.outputs().len(), 2),
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
