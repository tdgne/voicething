use super::*;
use crate::audio::stream::{node::NodeTrait, replicator::PeriodReplicator};
use imgui::*;

impl InputHandler for PeriodReplicator {}

impl PeriodReplicator {
    pub fn render(&mut self, ui: &Ui, state: &mut NodeEditorState) {
        let size = self.render_node(ui, state, "Period Replicator".to_string());

        let clicked = self.handle_input(ui, state, size);

        self.render_control_window(ui, state, clicked);
    }

    pub fn render_control_window(&mut self, ui: &Ui, state: &mut NodeEditorState, focused: bool) {
        let opened = state.window_opened(&self.id()).clone();
        if !opened {
            return;
        }
        let mouse_pos = ui.io().mouse_pos;
        Window::new(&im_str!("Period Replicator {:?}", self.id()))
            .opened(state.window_opened_mut(&self.id()))
            .focused(focused)
            .always_auto_resize(true)
            .position(mouse_pos, Condition::Once)
            .build(&ui, || {
                if ui.small_button(im_str!("Grab")) {
                    self.grab_period();
                }
                if ui.small_button(im_str!("Discard")) {
                    self.discard_period();
                }
            });
    }
}
