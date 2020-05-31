use crate::audio::common::Sample;
use crate::audio::stream::windower::Windower;
use crate::audio::node::HasId;
use super::NodeEditorState;
use imgui::*;
use super::*;

impl<S: Sample> InputHandler for Windower<S> {}

impl<S: Sample> Windower<S> {
    pub fn render_node(&mut self, ui: &Ui, state: &mut NodeEditorState) -> Option<ConnectRequest> {
        ui.set_cursor_pos([0.0, 0.0]);
        let win_pos = ui.cursor_screen_pos();
        let pos = state.pos(&self.id()).unwrap();
        let (w, h) = (100.0, 20.0);
        {
            let draw_list = ui.get_window_draw_list();
            let pos = [pos[0] + win_pos[0], pos[1] + win_pos[1]];
            draw_list
                .add_rect(pos, [pos[0] + w, pos[1] + h], (1.0, 1.0, 1.0, 1.0))
                .rounding(4.0)
                .filled(true)
                .build();
            draw_list.add_text(pos, (0.0, 0.0, 0.0, 1.0), "Windower");
        }

        let (focused, connection_request) = self.handle_input(ui, state, [w, h]);

        if focused {
            self.render_control_window(ui);
        }

        connection_request
    }

    pub fn render_control_window(&mut self, ui: &Ui) {
        Window::new(&im_str!("Window {}", self.id()))
            .always_auto_resize(true)
            .build(&ui, || {
                let mut window_size = *self.window_size_mut() as i32;
                Slider::new(
                    im_str!("window"),
                    std::ops::RangeInclusive::new(128, 2048),
                )
                .build(&ui, &mut window_size);
                *self.window_size_mut() = window_size as usize;
                let mut delay = *self.delay_mut() as i32;
                Slider::new(
                    im_str!("delay"),
                    std::ops::RangeInclusive::new(128, window_size),
                )
                .build(&ui, &mut delay);
                *self.delay_mut() = delay as usize;

            });

    }
}
