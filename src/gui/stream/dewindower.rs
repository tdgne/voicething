use super::NodeEditorState;
use super::*;
use crate::audio::stream::dewindower::Dewindower;
use crate::audio::stream::node::NodeTrait;
use imgui::*;

impl InputHandler for Dewindower {}

impl Dewindower {
    pub fn render(&mut self, ui: &Ui, state: &mut NodeEditorState) {
        let size = self.render_node(ui, state, "Dewindower".to_string());

        self.handle_input(ui, state, size);
    }

    pub fn render_control_window(&mut self, ui: &Ui) {}
}
