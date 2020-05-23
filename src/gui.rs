use std::thread;
use imgui::*;
mod support;

use crate::stream::{Mixer, EventReceiver, EventSender, ReceiverVolumePair, Runnable};

pub fn main_loop(input: EventReceiver<f32>, output: EventSender<f32>) {
    let system = support::init("voicething");

    thread::spawn(move || {
        loop {
            output.send(input.recv().unwrap()).unwrap();
        }
    });

    system.main_loop(move |_, ui| {
        Window::new(im_str!("voicething"))
            .size([300.0, 110.0], Condition::FirstUseEver)
            .build(ui, || {
                ui.text(im_str!("hi"));
            });
    });
}
