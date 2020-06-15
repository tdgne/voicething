use super::NodeEditorState;
use super::*;
use crate::audio::stream::{aggregate::*, node::NodeTrait};
use imgui::*;

impl InputHandler for AggregateNode {}

impl AggregateNode {
    pub fn render(&mut self, ui: &Ui, state: &mut NodeEditorState) {
        let size = self.render_node(ui, state, self.name().to_string());

        let clicked = self.handle_input(ui, state, size);

        self.render_control_window(ui, state, clicked);
    }

    fn name(&self) -> &str {
        match self.setting() {
            AggregateSetting::Sum => "Sum",
            AggregateSetting::Product => "Product",
        }
    }

    pub fn render_control_window(&mut self, ui: &Ui, state: &mut NodeEditorState, focused: bool) {
        let opened = state.window_opened(&self.id()).clone();
        if !opened {
            return;
        }
        let mouse_pos = ui.io().mouse_pos;
        Window::new(&im_str!("Aggregate {:?}", self.id()))
            .opened(state.window_opened_mut(&self.id()))
            .focused(focused)
            .always_auto_resize(true)
            .position(mouse_pos, Condition::Once)
            .build(&ui, || {
                ui.radio_button(im_str!("Sum"), self.setting_mut(), AggregateSetting::Sum);
                ui.radio_button(
                    im_str!("Product"),
                    self.setting_mut(),
                    AggregateSetting::Product,
                );
            });
    }
}
