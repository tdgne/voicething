use imgui::*;
use serde_json;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::io::{self, Read};

mod stream;
mod support;
use crate::audio;
use crate::audio::common::*;
use crate::audio::rechunker::Rechunker;
use crate::audio::stream::node::NodeTrait;
use crate::audio::stream::*;
use stream::*;

pub fn main_loop(host: audio::Host, input: Receiver<SampleChunk>, output: SyncSender<SampleChunk>) {
    let system = support::init("voicething");

    let rechunker = Arc::new(Mutex::new(Rechunker::new(2, 44100)));
    let (rechunk_tx, rechunk_rx) = sync_channel(32);
    thread::spawn(move || loop {
        rechunker.lock().unwrap().feed_chunk(input.recv().unwrap());
        while let Some(chunk) = rechunker.lock().unwrap().pull_chunk(1024) {
            rechunk_tx.try_send(chunk.clone()).unwrap();
        }
    });

    let mut node_editor_state = NodeEditorState::new();

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer);
    let mut g: Graph = match serde_json::from_str(&buffer) {
        Ok(g) => g,
        _ => {
            Graph::default()
        }
    };

    let (input_monitor_tx, input_monitor_rx) = sync_channel(32);
    let mut input_node = g.input_node().unwrap();
    let input_node_id = input_node.lock().unwrap().id();
    {
        let mut input_node = input_node.lock().unwrap();
        let mut input_port = input_node.add_input().unwrap();
        input_port.rx = Some(rechunk_rx);
    }
    {
        let mut input_node = input_node.lock().unwrap();
        let mut output_port = input_node.add_output().unwrap();
        output_port.tx = Some(input_monitor_tx);
    }
    node_editor_state.set_node_pos(input_node_id, [20.0, 20.0]);

    let (output_monitor_tx, output_monitor_rx) = sync_channel(32);
    let mut output_node = g.output_node().unwrap();
    let output_node_id = output_node.lock().unwrap().id();
    {
        let mut output_node = output_node.lock().unwrap();
        let mut output_port = output_node.add_output().unwrap();
        output_port.tx = Some(output_monitor_tx);
    }
    {
        let mut output_node = output_node.lock().unwrap();
        let mut output_port = output_node.add_output().unwrap();
        output_port.tx = Some(output);
    }
    node_editor_state.set_node_pos(output_node_id, [20.0, 100.0]);

    let g = Arc::new(Mutex::new(g));

    {
        let g = g.clone();
        thread::spawn(move || loop {
            g.lock().unwrap().run_once().unwrap();
            thread::sleep(std::time::Duration::from_millis(1));
        });
    }


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
                ui.menu(im_str!("File"), true, || {
                    if MenuItem::new(im_str!("Save")).build(&ui) {
                        println!("{}", serde_json::to_string(&*g.lock().unwrap()).unwrap());
                    }
                });
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
                ui.menu(im_str!("Nodes"), true, || {
                    let default_pos = [100.0, 100.0];
                    macro_rules! make_node_menu {
                        ($name:expr, $node:expr) => {
                            if MenuItem::new(im_str!($name)).build(&ui) {
                                let mut node = $node;
                                let node_id = node.id();
                                let mut g = g.lock().unwrap();
                                g.add(node);
                                g.add_input(&node_id).unwrap();
                                g.add_output(&node_id).unwrap();
                                node_editor_state.set_node_pos(node_id, default_pos);
                            }
                        }
                    }
                    make_node_menu!("TD-PSOLA", Node::Psola(PsolaNode::new(1.0)));
                    make_node_menu!("Windower", Node::Windower(Windower::new(WindowFunction::Hanning, 512, 64)));
                    make_node_menu!("Dewindower", Node::Dewindower(Dewindower::new(1024)));
                    make_node_menu!("Sum/Product", Node::Aggregate(AggregateNode::new(AggregateSetting::Sum)));
                    make_node_menu!("DFT/IDFT", Node::FourierTransform(FourierTransform::new(false, false)));
                    make_node_menu!("Arithmetic", Node::Arithmetic(ArithmeticNode::new(ArithmeticOperation::Log)));
                    make_node_menu!("Filter", Node::Filter(FilterNode::new(FilterOperation::ReplaceLowerAmplitudesFd{value: 0.0, threshold: 100.0})));
                    make_node_menu!("Monitor", Node::Identity(IdentityNode::new("Monitor".to_string())));
                });
            });
            Window::new(im_str!("I/O Monitor"))
                .always_auto_resize(true)
                .position([0.0, 20.0], Condition::FirstUseEver)
                .build(&ui, || {
                    while let Ok(SampleChunk::Real(chunk)) = input_rx.try_recv() {
                        input_amplitudes = chunk.samples(0).to_vec();
                    }
                    while let Ok(SampleChunk::Real(chunk)) = output_rx.try_recv() {
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
                            node.lock().unwrap().render(&ui, &mut node_editor_state);
                            for inputs in node.lock().unwrap().inputs().iter() {
                                connection_request = connection_request
                                    .or(inputs.render(&ui, &mut node_editor_state));
                            }
                            for outputs in node.lock().unwrap().outputs().iter() {
                                outputs.render(&ui, &mut node_editor_state);
                            }
                        }
                    }
                    if let Some(request) = connection_request {
                        let mut g = g.lock().unwrap();
                        if g.is_output_port(&request.0) && g.is_input_port(&request.1) {
                            let _ = g.connect_ports(&request.0, &request.1);
                        }
                    }
                    let draw_list = ui.get_window_draw_list();
                    ui.set_cursor_pos([0.0, 0.0]);
                    let win_pos = ui.cursor_screen_pos();
                    for (start, end) in g.lock().unwrap().edges().iter() {
                        let start_pos = node_editor_state.output_pos(start).unwrap();
                        let start_pos = [start_pos[0] + win_pos[0], start_pos[1] + win_pos[1]];
                        let end_pos = node_editor_state.input_pos(end).unwrap();
                        let end_pos = [end_pos[0] + win_pos[0], end_pos[1] + win_pos[1]];
                        draw_list
                            .add_line(start_pos.clone(), end_pos.clone(), (0.5, 0.5, 0.5, 0.5))
                            .thickness(2.0)
                            .build();
                    }
                });
        });
    }
}
