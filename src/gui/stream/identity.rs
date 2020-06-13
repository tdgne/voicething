use super::NodeEditorState;
use super::*;
use crate::audio::common::SampleChunk;
use crate::audio::stream::{identity::IdentityNode, node::NodeTrait};
use imgui::*;

impl InputHandler for IdentityNode {}

impl IdentityNode {
    pub fn render(&mut self, ui: &Ui, state: &mut NodeEditorState) {
        let size = self.render_node(ui, state, self.name().to_string());

        let clicked = self.handle_input(ui, state, size);

        self.render_control_window(ui, state, clicked);
    }

    pub fn render_control_window(&mut self, ui: &Ui, state: &mut NodeEditorState, focused: bool) {
        let opened = state.window_opened(&self.id()).clone();
        if !opened {
            return;
        }
        let mouse_pos = ui.io().mouse_pos;
        let amplitudes = match self.last_chunk() {
            Some(SampleChunk::Real(chunk)) => chunk.samples(0).to_vec(),
            Some(SampleChunk::Complex(chunk)) => chunk.samples(0).iter().map(|s| s.norm()).collect::<Vec<_>>(),
            _ => vec![],
        };
        Window::new(&im_str!("{} {:?}", self.name(), self.id()))
            .opened(state.window_opened_mut(&self.id()))
            .focused(focused)
            .always_auto_resize(true)
            .position(mouse_pos, Condition::Once)
            .build(&ui, || {
                ui.plot_lines(im_str!(""), &amplitudes)
                    .scale_max(1.0)
                    .graph_size([400.0, 300.0])
                    .build();
            });
    }
}
