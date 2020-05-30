use std::collections::HashMap;
use uuid::Uuid;
use crate::audio::stream::{SingleInput, Node};
use crate::audio::node::{sync_chunk_channel, HasId};
use std::sync::{Arc, Mutex};
use imgui::*;

pub type ConnectRequest = (Uuid, Uuid);

pub struct NodeEditorState {
    nodes: Arc<Mutex<Vec<Node>>>,
    graph: HashMap<Uuid, Vec<Uuid>>,
    node_pos: HashMap<Uuid, [f32; 2]>,
    left_dragged: Option<Uuid>,
    right_dragged: Option<Uuid>,
    focused: Option<Uuid>,
}

impl NodeEditorState {
    pub fn new(nodes: Arc<Mutex<Vec<Node>>>) -> Self {
        Self {
            nodes,
            graph: HashMap::new(),
            node_pos: HashMap::new(),
            left_dragged: None,
            right_dragged: None,
            focused: None,
        }
    }

    pub fn nodes(&self) -> Arc<Mutex<Vec<Node>>> {
        self.nodes.clone()
    }

    pub fn node(&self, uuid: Uuid) -> Option<Node> {
        for node in self.nodes.lock().unwrap().iter() {
            if node.id() == uuid {
                return Some(node.clone());
            }
        }
        None
    }

    pub fn graph(&self) ->  &HashMap<Uuid, Vec<Uuid>> {
        &self.graph
    }

    pub fn graph_mut(&mut self) ->  &mut HashMap<Uuid, Vec<Uuid>> {
        &mut self.graph
    }

    pub fn set_pos(&mut self, uuid: Uuid, pos: [f32; 2]) {
        self.node_pos.insert(uuid, pos);
    }

    pub fn pos(&self, uuid: &Uuid) -> Option<&[f32; 2]> {
        self.node_pos.get(uuid)
    }

    pub fn pos_mut(&mut self, uuid: &Uuid) -> Option<&mut [f32; 2]> {
        self.node_pos.get_mut(uuid)
    }

    pub fn set_left_dragged(&mut self, uuid: Option<Uuid>) {
        self.left_dragged = uuid;
    }

    pub fn left_dragged(&self) -> Option<Uuid> {
        self.left_dragged
    }

    pub fn set_right_dragged(&mut self, uuid: Option<Uuid>) {
        self.right_dragged = uuid;
    }

    pub fn right_dragged(&self) -> Option<Uuid> {
        self.right_dragged
    }

    pub fn set_focused(&mut self, uuid: Option<Uuid>) {
        self.focused = uuid
    }

    pub fn focused(&self) -> Option<Uuid> {
        self.focused
    }

    pub fn add_edge(&mut self, start: &Node, end: &Node) {
        for (_, v) in self.graph_mut().iter_mut() {
            *v = v.iter().filter(|n| **n != end.id()).cloned().collect::<Vec<_>>();
        }
        if let Some(v) = self.graph_mut().get_mut(&start.id()) {
            v.push(end.id());
        } else {
            self.graph_mut().insert(start.id(), vec![end.id()]);
        }
    }

    pub fn try_connect(&mut self, request: ConnectRequest) {
        use Node::*;
        let start = self.node(request.0).unwrap();
        let end = self.node(request.1).unwrap();
        match &start {
            Input(s) => 
                match &end {
                Input(_) => return,
                Output(e) => {
                    let (tx, rx) = sync_chunk_channel(1);
                    s.lock().unwrap().add_output(tx);
                    e.lock().unwrap().set_input(rx);
                    self.add_edge(&start, &end);
                },
                Psola(e) => {
                    let (tx, rx) = sync_chunk_channel(1);
                    s.lock().unwrap().add_output(tx);
                    e.lock().unwrap().set_input(rx);
                    self.add_edge(&start, &end);
                },
            },
            Psola(s) => match &end {
                Input(_) => return,
                Output(e) => {
                    let (tx, rx) = sync_chunk_channel(1);
                    s.lock().unwrap().add_output(tx);
                    e.lock().unwrap().set_input(rx);
                    self.add_edge(&start, &end);
                },
                Psola(e) => {
                    let (tx, rx) = sync_chunk_channel(1);
                    s.lock().unwrap().add_output(tx);
                    e.lock().unwrap().set_input(rx);
                    self.add_edge(&start, &end);
                },
            },
            Output(_) => return,
        }
    }
}

pub trait InputHandler: HasId {
    fn handle_input(&mut self, ui: &Ui, state: &mut NodeEditorState, size: [f32; 2]) -> (bool, Option<ConnectRequest>) {
        let win_pos = ui.cursor_screen_pos();
        let pos = state.pos(&self.id()).unwrap();
        let screen_pos = [pos[0] + win_pos[0], pos[1] + win_pos[1]];
        ui.set_cursor_screen_pos(screen_pos);
        let clicked = ui.invisible_button(&im_str!("{}", self.id()), size);
        let hovered = ui.is_item_hovered();

        // left drag
        let this_left_dragged = state.left_dragged() == Some(self.id());
        {
            let dragging = ui.is_mouse_dragging_with_threshold(MouseButton::Left, 2.0);
            if dragging && hovered {
                state.set_left_dragged(Some(self.id()));
            }
            if !dragging {
                state.set_left_dragged(None);
            }
            if dragging && this_left_dragged {
                let mouse_pos = ui.io().mouse_pos;
                let pos = [
                    mouse_pos[0] - win_pos[0] - size[0] / 2.0,
                    mouse_pos[1] - win_pos[1] - size[1] / 2.0,
                ];
                state.set_pos(self.id(), pos);
            }
        }

        // right drag
        let this_right_dragged = state.right_dragged() == Some(self.id());
        let mut connection_request = None;
        {
            let dragging = ui.is_mouse_dragging_with_threshold(MouseButton::Right, 2.0);
            if dragging && hovered && state.right_dragged().is_none() {
                state.set_right_dragged(Some(self.id()));
            }
            if !dragging {
                if let Some(start_node_id) = state.right_dragged() {
                    if hovered {
                        let end_node_id = self.id();
                        connection_request = Some((start_node_id, end_node_id));
                        state.set_right_dragged(None);
                    }
                }
            }
        };

        let truly_clicked = clicked && !this_left_dragged;
        if truly_clicked {
            state.set_focused(Some(self.id()));
        }

        let focused = state.focused() == Some(self.id());

        (focused, connection_request)
    }
}
