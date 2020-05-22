use imgui::*;
mod support;

use crate::stream::{Mixer};

pub fn main_loop(mixer: Mixer<f32>) {
    let system = support::init("voicething");

    system.main_loop(move |_, ui| {
        Window::new(im_str!("voicething"))
            .size([300.0, 110.0], Condition::FirstUseEver)
            .build(ui, || {
                ui.text(im_str!("hi"));
            });
    });
}
