use super::*;
use crate::audio::stream::{filter::*, node::NodeTrait};
use imgui::*;

#[derive(PartialEq, Copy, Clone, Debug)]
enum OpHighLow {
    High,
    Low,
}
#[derive(PartialEq, Copy, Clone, Debug)]
enum OpDomain {
    Time,
    Freq,
}

impl InputHandler for FilterNode {}

impl FilterNode {
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
            draw_list.add_text(pos, (0.0, 0.0, 0.0, 1.0), "Filter");
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
            return;
        }
        let mouse_pos = ui.io().mouse_pos;
        Window::new(&im_str!("Filter {:?}", self.id()))
            .opened(state.window_opened_mut(&self.id()))
            .focused(focused)
            .always_auto_resize(true)
            .position(mouse_pos, Condition::Once)
            .build(&ui, || {
                use FilterOperation::*;
                let mut op_hl = match self.op() {
                    ReplaceLowerAmplitudesFd { value, threshold } => OpHighLow::Low,
                    ReplaceLowerAmplitudesTd { value, threshold } => OpHighLow::Low,
                    _ => OpHighLow::High,
                };
                let mut op_d = match self.op() {
                    ReplaceLowerAmplitudesFd { value, threshold } => OpDomain::Freq,
                    ReplaceHigherAmplitudesFd { value, threshold } => OpDomain::Freq,
                    _ => OpDomain::Time,
                };
                let (mut threshold, mut value) = match self.op() {
                    ReplaceHigherAmplitudesFd { threshold, value } => (*threshold, *value),
                    ReplaceLowerAmplitudesFd { threshold, value } => (*threshold, *value),
                    ReplaceHigherAmplitudesTd { threshold, value } => (*threshold as f32, *value),
                    ReplaceLowerAmplitudesTd { threshold, value } => (*threshold as f32, *value),
                };
                ui.text("Replace region");
                ui.radio_button(im_str!("Low"), &mut op_hl, OpHighLow::Low);
                ui.radio_button(im_str!("High"), &mut op_hl, OpHighLow::High);
                ui.text("Domain");
                ui.radio_button(im_str!("Frequency"), &mut op_d, OpDomain::Freq);
                ui.radio_button(im_str!("Time"), &mut op_d, OpDomain::Time);
                Slider::new(
                    im_str!("Threshold"),
                    std::ops::RangeInclusive::new(0.0, 2000.0),
                )
                .display_format(im_str!("%0.2f"))
                .build(ui, &mut threshold);
                Slider::new(
                    im_str!("Amplitude"),
                    std::ops::RangeInclusive::new(0.0, 2.0),
                )
                .display_format(im_str!("%0.2f"))
                .build(ui, &mut value);
                *self.op_mut() = match op_hl {
                    OpHighLow::High => match op_d {
                        OpDomain::Time => ReplaceHigherAmplitudesTd {
                            threshold: threshold as usize,
                            value,
                        },
                        OpDomain::Freq => ReplaceHigherAmplitudesFd { threshold, value },
                    },
                    OpHighLow::Low => match op_d {
                        OpDomain::Time => ReplaceLowerAmplitudesTd {
                            threshold: threshold as usize,
                            value,
                        },
                        OpDomain::Freq => ReplaceLowerAmplitudesFd { threshold, value },
                    },
                };
            });
    }
}