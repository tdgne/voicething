use super::*;
use crate::audio::stream::{phasevocoder::PhaseVocoder, node::NodeTrait};
use imgui::*;

impl InputHandler for PhaseVocoder {}

impl PhaseVocoder {
    pub fn render(&mut self, ui: &Ui, state: &mut NodeEditorState) {
        let size = self.render_node(ui, state, "Phase Vocoder".to_string());

        let clicked = self.handle_input(ui, state, size);

        self.render_control_window(ui, state, clicked);
    }

    pub fn render_control_window(&mut self, ui: &Ui, state: &mut NodeEditorState, focused: bool) {
        let opened = state.window_opened(&self.id()).clone();
        if !opened {
            return
        }
        let mouse_pos = ui.io().mouse_pos;
        Window::new(&im_str!("Phase Vocoder {:?}", self.id()))
            .opened(state.window_opened_mut(&self.id()))
            .focused(focused)
            .always_auto_resize(true)
            .position(mouse_pos, Condition::Once)
            .build(&ui, || {
                VerticalSlider::new(
                    im_str!("pitch"),
                    [30.0, 200.0],
                    std::ops::RangeInclusive::new(0.5, 2.0),
                )
                .display_format(im_str!("%0.2f"))
                .build(&ui, self.rate_mut());
            });
    }
}
