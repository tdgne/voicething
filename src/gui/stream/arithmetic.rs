use super::*;
use crate::audio::stream::{
    arithmetic::{ArithmeticNode, ArithmeticOperation},
    node::NodeTrait,
};
use imgui::*;

impl InputHandler for ArithmeticNode {}

impl ArithmeticNode {
    pub fn render(&mut self, ui: &Ui, state: &mut NodeEditorState) {
        let size = self.render_node(ui, state, self.name().to_string());

        let clicked = self.handle_input(ui, state, size);

        self.render_control_window(ui, state, clicked);
    }

    fn name(&self) -> &str {
        match self.op() {
            ArithmeticOperation::Multiply(_) => "Multiply",
            ArithmeticOperation::Log => "Log",
            ArithmeticOperation::Exp => "Exp",
            ArithmeticOperation::Reciprocal => "Reciprocal",
            ArithmeticOperation::Inverse => "Inverse",
            ArithmeticOperation::Abs => "Abs",
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
                ui.radio_button(im_str!("Abs"), self.op_mut(), ArithmeticOperation::Abs);
            });
    }
}
