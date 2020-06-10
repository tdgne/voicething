use super::*;
use crate::audio::stream::{
    arithmetic::{ArithmeticNode, ArithmeticOperation},
    node::NodeTrait,
};
use imgui::*;

impl InputHandler for ArithmeticNode {}

impl ArithmeticNode {
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

        let clicked = self.handle_input(ui, state, [w, h]);

        self.render_control_window(ui, state, clicked);
    }

    fn name(&self) -> &str {
        match self.op() {
            ArithmeticOperation::Multiply(_) => "Multiply",
            ArithmeticOperation::Log => "Log",
            ArithmeticOperation::Exp => "Exp",
            ArithmeticOperation::Reciprocal => "Reciprocal",
            ArithmeticOperation::Inverse => "Inverse",
        }
    }

    pub fn render_control_window(&mut self, ui: &Ui, state: &mut NodeEditorState, focused: bool) {
        let opened = state.window_opened(&self.id()).clone();
        if !opened {
            return
        }
        let mouse_pos = ui.io().mouse_pos;
        Window::new(&im_str!("Arithmetic Node {:?}", self.id()))
            .opened(state.window_opened_mut(&self.id()))
            .focused(focused)
            .always_auto_resize(true)
            .position(mouse_pos, Condition::Once)
            .build(&ui, || {
                ui.radio_button(im_str!("Log"), self.op_mut(), ArithmeticOperation::Log);
                ui.radio_button(im_str!("Exp"), self.op_mut(), ArithmeticOperation::Exp);
                ui.radio_button(
                    im_str!("Reciprocal"),
                    self.op_mut(),
                    ArithmeticOperation::Reciprocal,
                );
                ui.radio_button(
                    im_str!("Inverse"),
                    self.op_mut(),
                    ArithmeticOperation::Inverse,
                );
            });
    }
}
