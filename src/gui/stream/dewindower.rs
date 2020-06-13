use crate::audio::stream::node::NodeTrait;
use crate::audio::stream::dewindower::Dewindower;
use super::NodeEditorState;
use imgui::*;
use super::*;

impl InputHandler for Dewindower {}

impl Dewindower {
    pub fn render(&mut self, ui: &Ui, state: &mut NodeEditorState) {
        let size = self.render_node(ui, state, "Dewindower".to_string());

        self.handle_input(ui, state, size);
    }

    pub fn render_control_window(&mut self, ui: &Ui) {
    }
}
