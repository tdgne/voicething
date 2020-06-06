use std::collections::HashMap;
use uuid::Uuid;
use crate::audio::stream::node::NodeTrait;
use imgui::*;

pub type ConnectRequest = (Uuid, Uuid);

pub struct NodeEditorState {
    node_pos: HashMap<Uuid, [f32; 2]>,
    input_pos: HashMap<Uuid, [f32; 2]>,
    output_pos: HashMap<Uuid, [f32; 2]>,
    left_dragged: Option<Uuid>,
    right_dragged: Option<Uuid>,
    focused: Option<Uuid>,
}

impl NodeEditorState {
    pub fn new() -> Self {
        Self {
            node_pos: HashMap::new(),
            input_pos: HashMap::new(),
            output_pos: HashMap::new(),
            left_dragged: None,
            right_dragged: None,
            focused: None,
        }
    }

    pub fn set_node_pos(&mut self, uuid: Uuid, pos: [f32; 2]) {
        self.node_pos.insert(uuid, pos);
    }

    pub fn set_input_pos(&mut self, uuid: Uuid, pos: [f32; 2]) {
        self.input_pos.insert(uuid, pos);
    }

    pub fn set_output_pos(&mut self, uuid: Uuid, pos: [f32; 2]) {
        self.output_pos.insert(uuid, pos);
    }

    pub fn node_pos(&self, uuid: &Uuid) -> Option<&[f32; 2]> {
        self.node_pos.get(uuid)
    }

    pub fn input_pos(&self, uuid: &Uuid) -> Option<&[f32; 2]> {
        self.input_pos.get(uuid)
    }

    pub fn output_pos(&self, uuid: &Uuid) -> Option<&[f32; 2]> {
        self.output_pos.get(uuid)
    }

    pub fn node_pos_mut(&mut self, uuid: &Uuid) -> Option<&mut [f32; 2]> {
        self.node_pos.get_mut(uuid)
    }

    pub fn set_left_dragged(&mut self, uuid: Option<Uuid>) {
        self.left_dragged = uuid;
    }

    pub fn left_dragged(&self) -> Option<Uuid> {
        self.left_dragged
    }

    pub fn set_right_dragged(&mut self, uuid: Option<Uuid>) {
        self.right_dragged = uuid;
    }

    pub fn right_dragged(&self) -> Option<Uuid> {
        self.right_dragged
    }

    pub fn set_focused(&mut self, uuid: Option<Uuid>) {
        self.focused = uuid
    }

    pub fn focused(&self) -> Option<Uuid> {
        self.focused
    }
}

pub trait InputHandler: NodeTrait {
    fn handle_input(&mut self, ui: &Ui, state: &mut NodeEditorState, size: [f32; 2]) -> bool {
        let win_pos = ui.cursor_screen_pos();
        let pos = state.node_pos(&self.id()).unwrap();
        let screen_pos = [pos[0] + win_pos[0], pos[1] + win_pos[1]];
        ui.set_cursor_screen_pos(screen_pos);
        let clicked = ui.invisible_button(&im_str!("{}", self.id()), size);
        let hovered = ui.is_item_hovered();

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

        let truly_clicked = clicked && !this_left_dragged;
        if truly_clicked {
            state.set_focused(Some(self.id()));
        }

        let focused = state.focused() == Some(self.id());

        focused
    }
}
