use imgui::*;
use std::sync::{Arc, Mutex};
use std::thread;

mod support;
use crate::audio;
use crate::audio::rechunker::Rechunker;
use crate::audio::stream::*;

pub fn main_loop(host: audio::Host, input: ChunkReceiver<f32>, output: ChunkSender<f32>) {
    let system = support::init("voicething");

    let rechunker = Arc::new(Mutex::new(Rechunker::new(2, 44100)));
    let (rechunk_main_tx, rechunk_main_rx) = chunk_channel();
    let (rechunk_monitor_tx, rechunk_monitor_rx) = chunk_channel();
    thread::spawn(move || loop {
        rechunker.lock().unwrap().feed_chunk(input.recv().unwrap());
        while let Some(chunk) = rechunker.lock().unwrap().pull_chunk(1024) {
            rechunk_main_tx.send(chunk.clone()).unwrap();
            rechunk_monitor_tx.send(chunk).unwrap();
        }
    });

    let (psola_monitor_tx, psola_monitor_rx) = chunk_channel();
    let mut psola = PsolaNode::new(1.0);
    psola.set_input(rechunk_main_rx);
    psola.add_output(output);
    psola.add_output(psola_monitor_tx);
    let psola = Arc::new(Mutex::new(psola));
    
    {
        let psola = psola.clone();
        thread::spawn(move || loop {
            psola.lock().unwrap().run_once();
            std::thread::sleep(std::time::Duration::from_millis(1));
        });
    }

    {
        let input_rx = rechunk_monitor_rx;
        let output_rx = psola_monitor_rx;
        let mut input_amplitudes = vec![];
        let mut output_amplitudes = vec![];
        system.main_loop(move |_, ui| {
            let current_input_device_name = host.current_input_device_name();
            let current_output_device_name = host.current_output_device_name();
            ui.main_menu_bar(|| {
                ui.menu(im_str!("Devices"), true, || {
                    ui.menu(im_str!("Input"), true, || {
                        for name in host.input_device_names().iter() {
                            let mut selected = current_input_device_name.clone().map(|n| n == *name).unwrap_or(false);
                            let was_selected = selected;
                            MenuItem::new(&im_str!("{}", name))
                                .build_with_ref(&ui, &mut selected);
                            if !was_selected && selected {
                                host.use_input_stream_from_device_name(name.clone());
                            }
                        }
                    });
                    ui.menu(im_str!("Output"), true, || {
                        for name in host.output_device_names().iter() {
                            let mut selected = current_output_device_name.clone().map(|n| n == *name).unwrap_or(false);
                            let was_selected = selected;
                            MenuItem::new(&im_str!("{}", name))
                                .build_with_ref(&ui, &mut selected);
                            if !was_selected && selected {
                                host.use_output_stream_from_device_name(name.clone());
                            }
                        }
                    });
                });
            });
            Window::new(im_str!("I/O Monitor"))
                .always_auto_resize(true)
                .position([0.0, 20.0], Condition::FirstUseEver)
                .build(&ui, || {
                    while let Ok(chunk) = input_rx.try_recv() {
                        input_amplitudes = chunk.samples(0).to_vec();
                    }
                    while let Ok(chunk) = output_rx.try_recv() {
                        output_amplitudes = chunk.samples(0).to_vec();
                    }
                    ui.plot_lines(im_str!(""), &input_amplitudes)
                        .overlay_text(im_str!("IN"))
                        .scale_min(-1.0)
                        .scale_max(1.0)
                        .graph_size([300.0, 100.0])
                        .build();
                    ui.plot_lines(im_str!(""), &output_amplitudes)
                        .overlay_text(im_str!("OUT"))
                        .scale_min(-1.0)
                        .scale_max(1.0)
                        .graph_size([300.0, 100.0])
                        .build();
                });
            Window::new(im_str!("TD-PSOLA"))
                .always_auto_resize(true)
                .position([400.0, 20.0], Condition::FirstUseEver)
                .build(&ui, || {
                    VerticalSlider::new(
                        im_str!("pitch"),
                        [30.0, 250.0],
                        std::ops::RangeInclusive::new(0.5, 2.0),
                    )
                    .display_format(im_str!("%0.2f"))
                    .build(&ui, psola.lock().unwrap().ratio_mut());
                });
        });
    }
}
