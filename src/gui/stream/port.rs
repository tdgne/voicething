use super::*;
use crate::audio::stream::node::{InputPort, OutputPort};
use imgui::*;

impl InputPort {
    pub fn render(&self, ui: &Ui, state: &mut NodeEditorState) -> Option<ConnectRequest> {
        ui.set_cursor_pos([0.0, 0.0]);
        let win_pos = ui.cursor_screen_pos();
        if let Some(pos) = state.input_pos(&self.id()) {
            let (w, h) = (5.0, 5.0);
            {
                let draw_list = ui.get_window_draw_list();
                let pos = [pos[0] + win_pos[0], pos[1] + win_pos[1]];
                draw_list
                    .add_triangle(
                        [pos[0] - w / 2.0, pos[1]],
                        [pos[0] + w / 2.0, pos[1]],
                        [pos[0], pos[1] + h],
                        (0.95, 0.95, 1.0, 1.0),
                    )
                    .filled(true)
                    .build();
            }
            self.handle_input(ui, state, [w, h])
        } else {
            None
        }
    }

    fn handle_input(
        &self,
        ui: &Ui,
        state: &mut NodeEditorState,
        size: [f32; 2],
    ) -> Option<ConnectRequest> {
        let win_pos = ui.cursor_screen_pos();
        let pos = state.input_pos(&self.id()).unwrap();
        let screen_pos = [pos[0] + win_pos[0] - size[0] / 2.0, pos[1] + win_pos[1]];
        ui.set_cursor_screen_pos(screen_pos);
        let clicked = ui.invisible_button(&im_str!("{:?}", self.id()), size);
        let hovered = ui.is_item_hovered();

        // right drag
        let mut connection_request = None;
        {
            let dragging = ui.is_mouse_dragging_with_threshold(MouseButton::Right, 0.0);
            if !dragging {
                if let Some(start_node_id) = state.right_dragged() {
                    if hovered {
                        let end_node_id = self.id();
                        connection_request = Some((start_node_id, end_node_id));
                        state.set_right_dragged(None);
                    }
                }
            }
        }

        connection_request
    }
}

impl OutputPort {
    pub fn render(&self, ui: &Ui, state: &mut NodeEditorState) {
        ui.set_cursor_pos([0.0, 0.0]);
        let win_pos = ui.cursor_screen_pos();
        if let Some(pos) = state.output_pos(&self.id()) {
            let (w, h) = (5.0, 5.0);
            {
                let draw_list = ui.get_window_draw_list();
                let pos = [pos[0] + win_pos[0], pos[1] + win_pos[1]];
                draw_list
                    .add_triangle(
                        [pos[0] - w / 2.0, pos[1]],
                        [pos[0] + w / 2.0, pos[1]],
                        [pos[0], pos[1] + h],
                        (0.95, 0.95, 1.0, 1.0),
                    )
                    .filled(true)
                    .build();
            }
            self.handle_input(ui, state, [w, h]);
        }
    }

    fn handle_input(&self, ui: &Ui, state: &mut NodeEditorState, size: [f32; 2]) {
        let win_pos = ui.cursor_screen_pos();
        let pos = state.output_pos(&self.id()).unwrap();
        let screen_pos = [pos[0] + win_pos[0] - size[0] / 2.0, pos[1] + win_pos[1]];
        ui.set_cursor_screen_pos(screen_pos);
        let clicked = ui.invisible_button(&im_str!("{:?}", self.id()), size);
        let hovered = ui.is_item_hovered();
        if hovered {
            ui.set_mouse_cursor(Some(MouseCursor::Hand));
        }

        // right drag
        let this_right_dragged = state.right_dragged() == Some(self.id());
        {
            let dragging = ui.is_mouse_dragging_with_threshold(MouseButton::Right, 0.0);
            if dragging && hovered && state.right_dragged().is_none() {
                state.set_right_dragged(Some(self.id()));
            }
        };
    }
}
