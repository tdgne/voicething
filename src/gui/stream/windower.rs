use crate::audio::stream::windower::Windower;
use crate::audio::stream::node::NodeTrait;
use super::NodeEditorState;
use imgui::*;
use super::*;

impl InputHandler for Windower {}

impl Windower {
    pub fn render(&mut self, ui: &Ui, state: &mut NodeEditorState) {
        let size = self.render_node(ui, state, "Windower".to_string());

        self.handle_input(ui, state, size);
    }

    pub fn render_control_window(&mut self, ui: &Ui) {
    }
}
