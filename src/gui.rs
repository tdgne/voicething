use imgui::*;
use std::thread;
use std::sync::{Arc, Mutex};
mod support;

use crate::stream::{
    EventReceiver, EventSender, Mixer, ProcessNode, PsolaNode, ReceiverVolumePair, Runnable,
};

pub fn main_loop(input: EventReceiver<f32>, output: EventSender<f32>) {
    let system = support::init("voicething");

    let psola = Arc::new(Mutex::new(PsolaNode::new(input, 1.5)));
    let psola_out = psola.lock().unwrap().output();

    thread::spawn(move || loop {
        output.send(psola_out.recv().unwrap()).unwrap();
    });

    thread::spawn(move || loop {
        psola.lock().unwrap().run_once();
    });

    system.main_loop(move |_, ui| {
        Window::new(im_str!("voicething"))
            .size([300.0, 110.0], Condition::FirstUseEver)
            .build(ui, || {
                ui.text(im_str!("hi"));
            });
    });
}
