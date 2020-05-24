use glium::{
    backend::Facade,
    texture::{ClientFormat, RawImage2d},
    Texture2d,
};
use imgui::*;
use rodio::DeviceTrait;
use std::sync::{Arc, Mutex};
use std::thread;

mod support;
use crate::config;
use crate::stream::{
    Event, EventReceiver, EventSender, Mixer, MultipleOutputNode, Multiplexer, PlaybackSink,
    ProcessNode, PsolaNode, ReceiverVolumePair, Runnable,
};

pub fn main_loop(
    input: EventReceiver<f32>,
    output: EventSender<f32>,
    playback_sink: Arc<Mutex<PlaybackSink>>,
    mut config: config::Config,
) {
    let system = support::init("voicething");

    let mut input_mtx = Multiplexer::new(input);
    let psola = Arc::new(Mutex::new(PsolaNode::new(input_mtx.new_output(), 1.0)));
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

    {
        let psola = psola.clone();
        thread::spawn(move || loop {
            psola.lock().unwrap().run_once();
            std::thread::sleep(std::time::Duration::from_millis(1));
        });
    }

    thread::spawn(move || {
        output_mtx.run();
    });

    let output_device_names = crate::audio::output_device_names();
    {
        let mut input_amplitudes = vec![];
        let mut output_amplitudes = vec![];
        system.main_loop(move |_, ui| {
            ui.main_menu_bar(|| {
                &ui.menu(im_str!("Output"), true, || {
                    for name in output_device_names.iter() {
                        let mut selected =
                            if let Some(config::Output::Device(used_name)) = config.output() {
                                *name == *used_name
                            } else {
                                false
                            };
                        let was_selected = selected;
                        MenuItem::new(&im_str!("{}", name)).build_with_ref(&ui, &mut selected);
                        if selected && !was_selected {
                            config.set_output(Some(config::Output::Device(name.clone())));
                            playback_sink
                                .lock()
                                .unwrap()
                                .set_rodio_sink(rodio::Sink::new(
                                    &rodio::output_devices()
                                        .unwrap()
                                        .filter(|d| d.name().unwrap() == *name)
                                        .next()
                                        .unwrap(),
                                ));
                        }
                    }
                });
            });
            Window::new(im_str!("I/O Monitor"))
                .always_auto_resize(true)
                .position([0.0, 0.0], Condition::FirstUseEver)
                .build(&ui, || {
                    if let Ok(Event::Chunk(chunk)) = input_mtx_out.try_recv() {
                        input_amplitudes = chunk.samples(0).to_vec();
                    }
                    if let Ok(Event::Chunk(chunk)) = output_mtx_out.try_recv() {
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
                .position([400.0, 0.0], Condition::FirstUseEver)
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
