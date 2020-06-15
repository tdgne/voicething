use super::*;
use crate::audio::stream::{
    formantshifter::{FormantShifter, Shift},
    node::NodeTrait,
};
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
            return;
        }
        let mouse_pos = ui.io().mouse_pos;
        let (w, h) = (400.0, 300.0);
        Window::new(&im_str!("Formant Shifter {:?}", self.id()))
            .opened(state.window_opened_mut(&self.id()))
            .focused(focused)
            .always_auto_resize(true)
            .position(mouse_pos, Condition::Once)
            .build(&ui, || {
                let cursor_pos = ui.cursor_screen_pos();
                ui.plot_lines(im_str!(""), self.prev_envelope())
                    .graph_size([w, h])
                    .build();
                let pos_x = mouse_pos[0] - cursor_pos[0];
                let d_f = self.prev_delta_f();
                let chunk_duration = self.prev_duration().unwrap_or(1024) as f32;
                let hovered = ui.is_item_hovered();
                let delta = ui.mouse_drag_delta_with_threshold(MouseButton::Left, 0.0)[0];
                if delta != 0.0 && ui.is_mouse_down(MouseButton::Left) && hovered {
                    let drag_to = pos_x / w * chunk_duration * d_f;
                    for shift in self.shifts_mut().iter_mut() {
                        if (shift.from - drag_to).abs() < 4.0 / w * chunk_duration * d_f {
                            shift.from = drag_to;
                        }
                        if (shift.to - drag_to).abs() < 4.0 / w * chunk_duration * d_f {
                            shift.to = drag_to;
                        }
                    }
                }
                let delta = ui.mouse_drag_delta_with_threshold(MouseButton::Right, 0.0)[0];
                if delta != 0.0 && ui.is_mouse_released(MouseButton::Right) && hovered {
                    let from = (pos_x - delta) / w * chunk_duration * d_f;
                    let to = pos_x / w * chunk_duration * d_f;
                    self.add_shift(Shift { from, to });
                }
                for shift in self.shifts().iter() {
                    let from_pos_x = shift.from / d_f / chunk_duration * w + cursor_pos[0];
                    let to_pos_x = shift.to / d_f / chunk_duration * w + cursor_pos[0];
                    let min_y = cursor_pos[1];
                    let max_y = cursor_pos[1] + h;
                    ui.get_window_draw_list()
                        .add_line([from_pos_x, min_y], [from_pos_x, max_y], (1.0, 0.0, 0.0))
                        .thickness(1.0)
                        .build();
                    ui.get_window_draw_list()
                        .add_line([to_pos_x, min_y], [to_pos_x, max_y], (0.0, 1.0, 0.0))
                        .thickness(1.0)
                        .build();
                    ui.get_window_draw_list()
                        .add_line(
                            [from_pos_x, (min_y + max_y) / 2.0],
                            [to_pos_x, (min_y + max_y) / 2.0],
                            (0.5, 0.5, 0.5, 0.5),
                        )
                        .thickness(1.0)
                        .build();
                }
            });
    }
}
