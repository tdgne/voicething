use imgui::*;
use std::sync::{Arc, Mutex};
use std::thread;
use serde_json;

mod stream;
mod support;
use crate::audio;
use crate::audio::rechunker::Rechunker;
use crate::audio::stream::*;
use stream::*;

pub fn main_loop(host: audio::Host, input: ChunkReceiver<f32>, output: SyncChunkSender<f32>) {
    let system = support::init("voicething");

    let rechunker = Arc::new(Mutex::new(Rechunker::new(2, 44100)));
    let (rechunk_tx, rechunk_rx) = chunk_channel();
    thread::spawn(move || loop {
        rechunker.lock().unwrap().feed_chunk(input.recv().unwrap());
        while let Some(chunk) = rechunker.lock().unwrap().pull_chunk(1024) {
            rechunk_tx.send(chunk.clone()).unwrap();
        }
    });

    let mut g = Graph::new();
    let mut node_editor_state = NodeEditorState::new();

    let (input_monitor_tx, input_monitor_rx) = sync_chunk_channel(16);
    let mut input_node = IdentityNode::new("Input".to_string());
    let input_node_id = input_node.id();
    input_node.set_input(Some(rechunk_rx));
    input_node.add_output(input_monitor_tx);
    g.add(Node::Input(input_node));
    node_editor_state.set_pos(input_node_id, [20.0, 20.0]);

    let psola_node = PsolaNode::new(1.0);
    let psola_node_id = psola_node.id();
    g.add(Node::Psola(psola_node));
    node_editor_state.set_pos(psola_node_id, [20.0, 60.0]);

    let (output_monitor_tx, output_monitor_rx) = sync_chunk_channel(16);
    let mut output_node = IdentityNode::new("Output".to_string());
    let output_node_id = output_node.id();
    output_node.add_output(output_monitor_tx);
    output_node.add_output(output);
    g.add(Node::Output(output_node));
    node_editor_state.set_pos(output_node_id, [20.0, 100.0]);

    g.connect(&input_node_id, &psola_node_id).unwrap();
    g.connect(&psola_node_id, &output_node_id).unwrap();

    let g = Arc::new(Mutex::new(g));

    {
        let g = g.clone();
        thread::spawn(move || loop {
            g.lock().unwrap().run_once().unwrap();
            thread::sleep(std::time::Duration::from_millis(1));
        });
    }

    println!("{}", serde_json::to_string(&*g.lock().unwrap()).unwrap());

    {
        let g = g.clone();
        let input_rx = input_monitor_rx;
        let output_rx = output_monitor_rx;
        let mut input_amplitudes = vec![];
        let mut output_amplitudes = vec![];
        system.main_loop(move |_, ui| {
            let current_input_device_name = host.current_input_device_name();
            let current_output_device_name = host.current_output_device_name();
            ui.main_menu_bar(|| {
                ui.menu(im_str!("Devices"), true, || {
                    ui.menu(im_str!("Input"), true, || {
                        for name in host.input_device_names().iter() {
                            let mut selected = current_input_device_name
                                .clone()
                                .map(|n| n == *name)
                                .unwrap_or(false);
                            let was_selected = selected;
                            MenuItem::new(&im_str!("{}", name)).build_with_ref(&ui, &mut selected);
                            if !was_selected && selected {
                                host.use_input_stream_from_device_name(name.clone());
                            }
                        }
                    });
                    ui.menu(im_str!("Output"), true, || {
                        for name in host.output_device_names().iter() {
                            let mut selected = current_output_device_name
                                .clone()
                                .map(|n| n == *name)
                                .unwrap_or(false);
                            let was_selected = selected;
                            MenuItem::new(&im_str!("{}", name)).build_with_ref(&ui, &mut selected);
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
            Window::new(im_str!("Nodes"))
                .position([400.0, 20.0], Condition::FirstUseEver)
                .size([600.0, 600.0], Condition::FirstUseEver)
                .build(&ui, || {
                    let mut connection_request = None;
                    {
                        for (_, node) in g.lock().unwrap().nodes().iter() {
                            match &mut *node.lock().unwrap() {
                                Node::Psola(node) => {
                                    connection_request = connection_request
                                        .or(node.render_node(&ui, &mut node_editor_state));
                                }
                                Node::Input(node) => {
                                    connection_request = connection_request
                                        .or(node.render_node(&ui, &mut node_editor_state));
                                }
                                Node::Output(node) => {
                                    connection_request = connection_request
                                        .or(node.render_node(&ui, &mut node_editor_state));
                                }
                            }
                        }
                    }
                    if let Some(request) = connection_request {
                        let _ = g.lock().unwrap().connect(&request.0, &request.1);
                    }
                    let draw_list = ui.get_window_draw_list();
                    ui.set_cursor_pos([0.0, 0.0]);
                    let win_pos = ui.cursor_screen_pos();
                    for (start, ends) in g.lock().unwrap().edges().iter() {
                        let start_pos = node_editor_state.pos(start).unwrap();
                        let start_pos = [start_pos[0] + win_pos[0], start_pos[1] + win_pos[1]];
                        for end in ends {
                            let end_pos = node_editor_state.pos(end).unwrap();
                            let end_pos = [end_pos[0] + win_pos[0], end_pos[1] + win_pos[1]];
                            draw_list
                                .add_line(start_pos.clone(), end_pos.clone(), (0.5, 0.5, 0.5, 0.5))
                                .thickness(2.0)
                                .build();
                        }
                    }
                });
        });
    }
}
