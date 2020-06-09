use super::*;
use imgui::*;
use crate::audio::stream::node::*;

impl Node {
    pub fn render(&mut self, ui: &Ui, node_editor_state: &mut NodeEditorState) {
        match self {
            Node::Psola(node) => {
                node.render(ui, node_editor_state);
            }
            Node::Identity(node) => {
                node.render(ui, node_editor_state);
            }
            Node::Windower(node) => {
                node.render(ui, node_editor_state);
            }
            Node::Dewindower(node) => {
                node.render(ui, node_editor_state);
            }
            Node::Aggregate(node) => {
                node.render(ui, node_editor_state);
            }
            Node::FourierTransform(node) => {
                node.render(ui, node_editor_state);
            }
            Node::Arithmetic(node) => {
                node.render(ui, node_editor_state);
            }
        }
    }
}
