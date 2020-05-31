use super::node::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::{
    fmt,
    fmt::{Display, Formatter},
};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Graph {
    nodes: HashMap<Uuid, Arc<Mutex<Node>>>,
    edges: HashMap<Uuid, Vec<Uuid>>,
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
        }
    }

    fn bfs_run_once(&self, node: Arc<Mutex<Node>>) -> Result<(), Box<dyn Error>> {
        let mut q = VecDeque::new();
        q.push_back(node);
        while !q.is_empty() {
            let node = q.pop_front().unwrap();
            node.lock().unwrap().run_once();
            for output in self
                .edges
                .get(&node.lock().unwrap().id())
                .unwrap_or(&vec![])
                .iter()
            {
                q.push_back(self.node(output).unwrap())
            }
        }
        Ok(())
    }

    pub fn run_once(&self) -> Result<(), Box<dyn Error>> {
        let input = {
            let mut input = Err(ExistenceError("Input"));
            for (_, v) in self.nodes.iter() {
                if let Node::Input(_) = &*v.lock().unwrap() {
                    input = Ok(v.clone());
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

    pub fn nodes(&self) -> &HashMap<Uuid, Arc<Mutex<Node>>> {
        &self.nodes
    }

    pub fn edges(&self) -> &HashMap<Uuid, Vec<Uuid>> {
        &self.edges
    }

    pub fn add(&mut self, node: Node) {
        self.edges.insert(node.id(), vec![]);
        self.nodes.insert(node.id(), Arc::new(Mutex::new(node)));
    }

    pub fn remove(&mut self, id: Uuid) -> Option<Arc<Mutex<Node>>> {
        self.detach(id);
        self.nodes.remove(&id)
    }

    pub fn node(&self, id: &Uuid) -> Result<Arc<Mutex<Node>>, ExistenceError> {
        if let Some(ref node) = self.nodes.get(&id) {
            Ok(Arc::clone(node))
        } else {
            Err(ExistenceError("node"))
        }
    }

    pub fn connect(&mut self, from_id: &Uuid, to_id: &Uuid) -> Result<(), Box<dyn Error>> {
        if *from_id == *to_id {
            // TODO
            return Ok(());
        }
        let mut detach_id = None;
        use Node::*;
        {
            let from_rc = self.node(from_id)?;
            let from = from_rc.lock().unwrap();
            let to_rc = self.node(to_id)?;
            let to = to_rc.lock().unwrap();
            let cerr = Err(CompatibilityError);
            match &*from {
                Input(_) => match &*to {
                    Input(_) => cerr?,
                    Output(_) => {
                        detach_id = Some(*to_id);
                    }
                    Psola(_) => {
                        detach_id = Some(*to_id);
                    }
                    Windower(_) => {
                        detach_id = Some(*to_id);
                    }
                },
                Psola(_) => match &*to {
                    Input(_) => cerr?,
                    Output(_) => {
                        detach_id = Some(*to_id);
                    }
                    Psola(_) => {
                        detach_id = Some(*to_id);
                    }
                    Windower(_) => {
                        detach_id = Some(*to_id);
                    }
                },
                Windower(_) => match &*to {
                    Psola(_) => {
                        detach_id = Some(*to_id);
                    }
                    _ => cerr?,
                },
                Output(_) => cerr?,
            }
        }
        if let Some(id) = detach_id {
            self.detach(id)?;
        }
        {
            let from_rc = self.node(from_id)?;
            let mut from = from_rc.lock().unwrap();
            let to_rc = self.node(to_id)?;
            let mut to = to_rc.lock().unwrap();
            let cerr = Err(CompatibilityError);
            let (tx, rx) = sync_chunk_channel(16);
            match &mut *from {
                Input(ref mut s) => match &mut *to {
                    Input(_) => cerr?,
                    Output(ref mut e) => {
                        s.add_output(tx);
                        e.set_input(Some(rx));
                    },
                    Psola(ref mut e) => {
                        s.add_output(tx);
                        e.set_input(Some(rx));
                    },
                    Windower(ref mut e) => {
                        s.add_output(tx);
                        e.set_input(Some(rx));
                    },
                },
                Psola(ref mut s) => match &mut *to {
                    Input(_) => cerr?,
                    Output(ref mut e) => {
                        s.add_output(tx);
                        e.set_input(Some(rx));
                    },
                    Psola(ref mut e) => {
                        s.add_output(tx);
                        e.set_input(Some(rx));
                    },
                    Windower(ref mut e) => {
                        s.add_output(tx);
                        e.set_input(Some(rx));
                    },
                },
                Windower(ref mut s) => match &mut *to {
                    Psola(ref mut e) => {
                        s.add_output(tx);
                        e.set_input(Some(rx));
                    },
                    _ => cerr?,
                },
                Output(_) => cerr?,
            }
        }
        self.connect_edge(*from_id, *to_id)?;
        Ok(())
    }

    pub fn disconnect(&mut self, from_id: &Uuid, to_id: &Uuid) -> Result<(), Box<dyn Error>> {
        let from_rc = self.node(from_id)?;
        let mut from = from_rc.lock().unwrap();
        let to_rc = self.node(to_id)?;
        let mut to = to_rc.lock().unwrap();
        let cerr = Err(CompatibilityError);
        use Node::*;
        match &mut *from {
            Input(_) => match &mut *to {
                Input(_) => cerr?,
                Output(ref mut e) => {
                    e.set_input(None);
                    self.disconnect_edge(*from_id, *to_id)?;
                }
                Psola(ref mut e) => {
                    e.set_input(None);
                    self.disconnect_edge(*from_id, *to_id)?;
                }
                Windower(ref mut e) => {
                    e.set_input(None);
                    self.disconnect_edge(*from_id, *to_id)?;
                }
            },
            Psola(_) => match &mut *to {
                Input(_) => cerr?,
                Output(ref mut e) => {
                    e.set_input(None);
                    self.disconnect_edge(*from_id, *to_id)?;
                }
                Psola(ref mut e) => {
                    e.set_input(None);
                    self.disconnect_edge(*from_id, *to_id)?;
                }
                Windower(ref mut e) => {
                    e.set_input(None);
                    self.disconnect_edge(*from_id, *to_id)?;
                }
            },
            Windower(_) => match &mut *to {
                _ => cerr?,
                Psola(ref mut e) => {
                    e.set_input(None);
                    self.disconnect_edge(*from_id, *to_id)?;
                }
            },
            Output(_) => cerr?,
        }
        Ok(())
    }

    pub fn detach(&mut self, id: Uuid) -> Result<(), Box<dyn Error>> {
        for (from_id, tos) in self.edges.clone().iter() {
            if *from_id == id {
                for to_id in tos.iter() {
                    self.disconnect(&id, to_id)?;
                }
            } else {
                for _ in tos.iter().filter(|to_id| **to_id == id) {
                    self.disconnect(from_id, &id)?;
                }
            }
        }
        Ok(())
    }

    fn connect_edge(&mut self, from_id: Uuid, to_id: Uuid) -> Result<(), Box<dyn Error>> {
        if let Some(edges) = self.edges.get_mut(&from_id) {
            let _ = edges.remove_item(&to_id);
            edges.push(to_id);
        } else {
            self.edges.insert(from_id, vec![to_id]);
        }
        Ok(())
    }

    fn disconnect_edge(&mut self, from_id: Uuid, to_id: Uuid) -> Result<(), ExistenceError> {
        if let Some(edges) = self.edges.get_mut(&from_id) {
            if let None = edges.remove_item(&to_id) {
                return Err(ExistenceError("edge"));
            }
        } else {
            return Err(ExistenceError("edge"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::super::*;
    #[test]
    fn connect_run_disconnect() {
        let mut g = Graph::new();
        let n1 = Node::Input(IdentityNode::new("Input".to_string()));
        let n2 = Node::Psola(PsolaNode::new(1.0));
        let n1_id = n1.id();
        let n2_id = n2.id();
        g.add(n1);
        g.add(n2);
        g.connect(&n1_id, &n2_id).unwrap();
        assert_eq!(g.edges().get(&n1_id).unwrap()[0], n2_id);
        match &*g.node(&n1_id).unwrap().lock().unwrap() {
            Node::Input(ref n) => assert_eq!(n.outputs().len(), 1),
            _ => panic!(),
        };
        match &*g.node(&n2_id).unwrap().lock().unwrap() {
            Node::Psola(ref n) => assert!(n.input().is_some()),
            _ => panic!(),
        };
        g.run_once().unwrap();
        g.disconnect(&n1_id, &n2_id).unwrap();
        assert_eq!(g.edges().get(&n1_id).unwrap().len(), 0);
        match &*g.node(&n2_id).unwrap().lock().unwrap() {
            Node::Psola(ref n) => assert!(n.input().is_none()),
            _ => panic!(),
        };
    }
}
