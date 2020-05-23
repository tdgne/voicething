use imgui::*;
use glium::{
    backend::Facade,
    texture::{ClientFormat, RawImage2d},
    Texture2d,
};
use std::thread;
use std::sync::{Arc, Mutex};
mod support;

use crate::stream::{
    MultipleOutputNode, EventReceiver, EventSender, Mixer, ProcessNode, PsolaNode, ReceiverVolumePair, Runnable, Multiplexer, Event
};

pub fn main_loop(input: EventReceiver<f32>, output: EventSender<f32>) {
    let system = support::init("voicething");

    let mut input_mtx = Multiplexer::new(input);
    let psola = Arc::new(Mutex::new(PsolaNode::new(input_mtx.new_output(), 1.5)));
    let psola_out = psola.lock().unwrap().output();
    let mut output_mtx = Multiplexer::new(psola_out);

    {
        let output_mtx_out = output_mtx.new_output();
        thread::spawn(move || loop {
            output.send(output_mtx_out.recv().unwrap()).unwrap();
        });
    }

    let input_mtx_out = input_mtx.new_output();
    let output_mtx_out = output_mtx.new_output();

    thread::spawn(move || {
        input_mtx.run();
    });

    thread::spawn(move || loop {
        psola.lock().unwrap().run_once();
    });

    thread::spawn(move || {
        output_mtx.run();
    });

    {
        let mut input_values = vec![];
        let mut output_values = vec![];
        let mut time = 0u32;
        system.main_loop(move |_, ui| {
            Window::new(im_str!("waveforms"))
                .size([500.0, 300.0], Condition::FirstUseEver)
                .build(ui, || {
                    if let Ok(Event::Chunk(chunk)) = input_mtx_out.try_recv() {
                        input_values.push(chunk.samples(0).to_vec());
                    }
                    if let Ok(Event::Chunk(chunk)) = output_mtx_out.try_recv() {
                        output_values.push(chunk.samples(0).to_vec());
                    }
                    let display_input_values = if time > 0 && time < input_values.len() as u32 {
                        Some(&input_values[time as usize])
                    } else {
                        None
                    };
                    let display_output_values = if time > 0 && time < output_values.len() as u32 {
                        Some(&output_values[time as usize])
                    } else {
                        None
                    };
                    if let Some(d) = display_input_values {
                    let p = &input_values[time as usize-1];
                        ui.plot_lines(im_str!("input"), &[&p[..], &d[..]].concat())
                            .scale_min(-1.0)
                            .scale_max(1.0)
                            .graph_size([400.0, 100.0])
                            .build();
                    }
                    if let Some(d) = display_output_values {
                    let p = &output_values[time as usize-1];
                        ui.plot_lines(im_str!("output"), &[&p[..], &d[..]].concat())
                            .scale_min(-1.0)
                            .scale_max(1.0)
                            .graph_size([400.0, 100.0])
                            .build();
                    }
                    Slider::new(im_str!(""), std::ops::RangeInclusive::new(0u32, input_values.len() as u32))
                        .build(&ui, &mut time);
                });
        });
    }
}
