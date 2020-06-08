use super::*;
use crate::audio::stream::{node::NodeTrait, ft::FourierTransform};
use imgui::*;

impl InputHandler for FourierTransform {}

impl FourierTransform {
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
            draw_list.add_text(pos, (0.0, 0.0, 0.0, 1.0), format!("{}", self.name()));
        }

        state.set_input_pos(self.inputs()[0].id(), [pos[0], pos[1] - 5.0]);
        for (i, output) in self.outputs().iter().enumerate() {
            state.set_output_pos(output.id(), [pos[0] + 10.0 * i as f32, pos[1] + h]);
        }

        let focused = self.handle_input(ui, state, [w, h]);

        if focused {
            self.render_control_window(ui);
        }
    }

    fn name(&self) -> &str {
        if self.inverse() {
            "DFT"
        } else {
            "IDFT"
        }
    }

    pub fn render_control_window(&mut self, ui: &Ui) {
        Window::new(&im_str!("{} {:?}", self.name(), self.id()))
            .always_auto_resize(true)
            .build(&ui, || {
                ui.checkbox(im_str!("inverse"), self.inverse_mut());
                if self.inverse() {
                    ui.checkbox(im_str!("real output"), self.real_output_mut());
                }
            });
    }
}
