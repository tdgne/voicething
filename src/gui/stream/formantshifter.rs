use super::*;
use crate::audio::stream::{formantshifter::FormantShifter, node::NodeTrait};
use imgui::*;

impl InputHandler for FormantShifter {}

impl FormantShifter {
    pub fn render(&mut self, ui: &Ui, state: &mut NodeEditorState) {
        let size = self.render_node(ui, state, "Formant Shifter".to_string());

        let clicked = self.handle_input(ui, state, size);

        self.render_control_window(ui, state, clicked);
    }

    pub fn render_control_window(&mut self, ui: &Ui, state: &mut NodeEditorState, focused: bool) {
        let opened = state.window_opened(&self.id()).clone();
        if !opened {
            return
        }
        let mouse_pos = ui.io().mouse_pos;
        Window::new(&im_str!("Formant Shifter {:?}", self.id()))
            .opened(state.window_opened_mut(&self.id()))
            .focused(focused)
            .always_auto_resize(true)
            .position(mouse_pos, Condition::Once)
            .build(&ui, || {
                ui.text("test");
            });
    }
}
