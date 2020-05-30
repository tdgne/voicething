use crate::audio::common::Sample;
use crate::audio::stream::identity::IdentityNode;
use crate::audio::node::HasId;
use super::NodeEditorState;
use imgui::*;
use super::*;

impl<S: Sample> InputHandler for IdentityNode<S> {}

impl<S: Sample> IdentityNode<S> {
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
            draw_list.add_text(pos, (0.0, 0.0, 0.0, 1.0), self.name());
        }

        self.handle_input(ui, state, [w, h]).1
    }

    pub fn render_control_window(&mut self, ui: &Ui) {
    }
}
