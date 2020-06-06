use crate::audio::stream::windower::Windower;
use crate::audio::stream::node::NodeTrait;
use super::NodeEditorState;
use imgui::*;
use super::*;

impl InputHandler for Windower {}

impl Windower {
    pub fn render(&mut self, ui: &Ui, state: &mut NodeEditorState) {
        ui.set_cursor_pos([0.0, 0.0]);
        let win_pos = ui.cursor_screen_pos();
        let pos = state.node_pos(&self.id()).unwrap();
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

        self.handle_input(ui, state, [w, h]);
    }

    pub fn render_control_window(&mut self, ui: &Ui) {
    }
}
