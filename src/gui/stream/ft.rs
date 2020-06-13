use super::*;
use crate::audio::stream::{ft::FourierTransform, node::NodeTrait};
use imgui::*;

impl InputHandler for FourierTransform {}

impl FourierTransform {
    pub fn render(&mut self, ui: &Ui, state: &mut NodeEditorState) {
        let size = self.render_node(ui, state, self.name().to_string());

        let clicked = self.handle_input(ui, state, size);

        self.render_control_window(ui, state, clicked);
    }

    fn name(&self) -> &str {
        if self.inverse() {
            "IDFT"
        } else {
            "DFT"
        }
    }

    pub fn render_control_window(&mut self, ui: &Ui, state: &mut NodeEditorState, focused: bool) {
        let opened = state.window_opened(&self.id()).clone();
        if !opened {
            return
        }
        let mouse_pos = ui.io().mouse_pos;
        Window::new(&im_str!("Fourier Transform {:?}", self.id()))
            .opened(state.window_opened_mut(&self.id()))
            .focused(focused)
            .always_auto_resize(true)
            .position(mouse_pos, Condition::Once)
            .build(&ui, || {
                ui.checkbox(im_str!("inverse"), self.inverse_mut());
                if self.inverse() {
                    ui.checkbox(im_str!("real output"), self.real_output_mut());
                }
            });
    }
}
