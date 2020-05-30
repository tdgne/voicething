use super::*;
use crate::audio::stream::node::HasId;
use crate::audio::stream::psola::PsolaNode;
use imgui::*;

impl InputHandler for PsolaNode {}

impl PsolaNode {
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
            draw_list.add_text(pos, (0.0, 0.0, 0.0, 1.0), "TD-PSOLA");
        }

        let (focused, connection_request) = self.handle_input(ui, state, [w, h]);

        if focused {
            self.render_control_window(ui);
        }

        connection_request
    }

    pub fn render_control_window(&mut self, ui: &Ui) {
        Window::new(&im_str!("TD-PSOLA {}", self.id()))
            .always_auto_resize(true)
            .build(&ui, || {
                VerticalSlider::new(
                    im_str!("pitch"),
                    [30.0, 200.0],
                    std::ops::RangeInclusive::new(0.5, 2.0),
                )
                .display_format(im_str!("%0.2f"))
                .build(&ui, self.ratio_mut());
            });
    }
}
