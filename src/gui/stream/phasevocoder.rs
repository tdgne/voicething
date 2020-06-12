use super::*;
use crate::audio::stream::{phasevocoder::PhaseVocoder, node::NodeTrait};
use imgui::*;

impl InputHandler for PhaseVocoder {}

impl PhaseVocoder {
    pub fn render(&mut self, ui: &Ui, state: &mut NodeEditorState) {
        ui.set_cursor_pos([0.0, 0.0]);
        let win_pos = ui.cursor_screen_pos();
        let pos = state.node_pos(&self.id()).unwrap().clone();
        let (w, h) = (100.0, 20.0);
        {
            let draw_list = ui.get_window_draw_list();
            let pos = [pos[0] + win_pos[0], pos[1] + win_pos[1]];
            draw_list
                .add_rect(pos, [pos[0] + w, pos[1] + h], (1.0, 1.0, 1.0, 1.0))
                .rounding(4.0)
                .filled(true)
                .build();
            draw_list.add_text(pos, (0.0, 0.0, 0.0, 1.0), format!("Phase Vocoder"));
        }

        state.set_input_pos(self.inputs()[0].id(), [pos[0], pos[1] - 5.0]);
        for (i, output) in self.outputs().iter().enumerate() {
            state.set_output_pos(output.id(), [pos[0] + 10.0 * i as f32, pos[1] + h]);
        }

        let clicked = self.handle_input(ui, state, [w, h]);

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
