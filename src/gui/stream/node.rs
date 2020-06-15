use super::*;
use crate::audio::stream::node::*;
use imgui::*;

impl Node {
    pub fn render(&mut self, ui: &Ui, node_editor_state: &mut NodeEditorState) {
        match self {
            Node::Psola(node) => {
                node.render(ui, node_editor_state);
            }
            Node::Identity(node) => {
                node.render(ui, node_editor_state);
            }
            Node::Windower(node) => {
                node.render(ui, node_editor_state);
            }
            Node::Dewindower(node) => {
                node.render(ui, node_editor_state);
            }
            Node::Aggregate(node) => {
                node.render(ui, node_editor_state);
            }
            Node::FourierTransform(node) => {
                node.render(ui, node_editor_state);
            }
            Node::Arithmetic(node) => {
                node.render(ui, node_editor_state);
            }
            Node::Filter(node) => {
                node.render(ui, node_editor_state);
            }
            Node::PhaseVocoder(node) => {
                node.render(ui, node_editor_state);
            }
            Node::PeriodReplicator(node) => {
                node.render(ui, node_editor_state);
            }
            Node::FormantShifter(node) => {
                node.render(ui, node_editor_state);
            }
        }
    }
}

pub trait InputHandler: NodeTrait {
    fn render_node(&mut self, ui: &Ui, state: &mut NodeEditorState, name: String) -> [f32; 2] {
        ui.set_cursor_pos([0.0, 0.0]);
        let win_pos = ui.cursor_screen_pos();
        let pos = state.node_pos(&self.id()).unwrap().clone();
        let text_size = ui.calc_text_size(&im_str!("{}", name), false, 1000.0);
        let (padding_x, padding_y) = (10.0, 5.0);
        let (w, h) = (text_size[0] + 2.0 * padding_x, text_size[1] + 2.0 * padding_y);
        {
            let draw_list = ui.get_window_draw_list();
            let pos = [pos[0] + win_pos[0], pos[1] + win_pos[1]];
            draw_list
                .add_rect(pos, [pos[0] + w, pos[1] + h], (0.9, 0.9, 1.0, 0.8))
                .rounding(4.0)
                .filled(true)
                .build();
            draw_list.add_text([pos[0] + w / 2.0 - text_size[0] / 2.0, pos[1] + padding_y], (0.0, 0.0, 0.0, 1.0), name);
        }

        fn divide(w: f32, l: usize, i: usize) -> f32 {
            (w / (l as f32 + 1.0)) * (i + 1) as f32
        }

        let mut i = 0;
        let l = self.inputs().len();
        for input in self.inputs().iter() {
            if !(input.rx.is_some() && input.output_id.is_none()) {
                state.set_input_pos(input.id(), [pos[0] + divide(w, l, i), pos[1] - 5.0]);
                i += 1;
            }
        }

        let mut i = 0;
        let l = self.outputs().len();
        for output in self.outputs().iter() {
            if !(output.tx.is_some() && output.input_id.is_none()) {
                state.set_output_pos(output.id(), [pos[0] + divide(w, l, i), pos[1] + h]);
                i += 1;
            }
        }
        [w, h]
    }

    fn handle_input(&mut self, ui: &Ui, state: &mut NodeEditorState, size: [f32; 2]) -> bool {
        let win_pos = ui.cursor_screen_pos();
        let pos = state.node_pos(&self.id()).unwrap();
        let screen_pos = [pos[0] + win_pos[0], pos[1] + win_pos[1]];
        ui.set_cursor_screen_pos(screen_pos);
        let clicked = ui.invisible_button(&im_str!("{:?}", self.id()), size);
        let hovered = ui.is_item_hovered();
        let double_clicked = hovered && ui.is_mouse_double_clicked(MouseButton::Left);

        // left drag
        let this_left_dragged = state.left_dragged() == Some(self.id());
        {
            let dragging = ui.is_mouse_dragging_with_threshold(MouseButton::Left, 2.0);
            if dragging && hovered {
                state.set_left_dragged(Some(self.id()));
            }
            if !dragging {
                state.set_left_dragged(None);
            }
            if dragging && this_left_dragged {
                let mouse_pos = ui.io().mouse_pos;
                let pos = [
                    mouse_pos[0] - win_pos[0] - size[0] / 2.0,
                    mouse_pos[1] - win_pos[1] - size[1] / 2.0,
                ];
                state.set_node_pos(self.id(), pos);
            }
        }

        if double_clicked {
            *state.window_opened_mut(&self.id()) = true;
        }

        double_clicked
    }
}
