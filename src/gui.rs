use glium::{
    backend::Facade,
    texture::{ClientFormat, RawImage2d},
    Texture2d,
};
use imgui::*;
use std::sync::{Arc, Mutex};
use std::thread;

mod support;
use crate::audio;
use crate::common::AudioMetadata;
use crate::stream::{
    Event, EventReceiver, EventSender, Mixer, MultipleOutputNode, Multiplexer, PlaybackSink,
    ProcessNode, PsolaNode, ReceiverVolumePair, Runnable, SingleOutputNode,
};

pub fn main_loop(
    mut input: EventReceiver<f32>,
    mut output: EventSender<f32>,
    mut audio_state: audio::AudioState,
) {
    let input_device_names = audio::INPUT_DEVICE_NAMES.lock().unwrap().clone();
    let output_device_names = audio::OUTPUT_DEVICE_NAMES.lock().unwrap().clone();

    let audio_config = audio_state.config().clone();
    let system = support::init("voicething");

    let input_mtx = Arc::new(Mutex::new(Multiplexer::new(input)));
    let psola = Arc::new(Mutex::new(PsolaNode::new(
        input_mtx.lock().unwrap().new_output(),
        1.0,
    )));
    let psola_out = psola.lock().unwrap().output();

    let mut mixer = Mixer::new(
        vec![ReceiverVolumePair {
            receiver: psola_out,
            volume: 1.0,
        }],
        AudioMetadata::new(2, audio_state.output_sample_rate().unwrap()),
        (*audio_config.chunk_size() as f32 * audio_state.output_sample_rate().unwrap() as f32
            / audio_state.input_sample_rate().unwrap_or(22050) as f32)
            .ceil() as usize, // give a decent maximum sample duration for the buffer
    );
    let mixer_out = mixer.output();
    thread::spawn(move || {
        mixer.run();
    });

    let mut output_mtx = Multiplexer::new(mixer_out);

    {
        let output_mtx_out = output_mtx.new_output();
        thread::spawn(move || loop {
            output.send(output_mtx_out.recv().unwrap()).unwrap();
        });
    }

    let input_mtx_out = input_mtx.lock().unwrap().new_output();
    let output_mtx_out = output_mtx.new_output();

    {
        let input_mtx = input_mtx.clone();
        thread::spawn(move || loop {
            input_mtx.lock().unwrap().run_once();
            std::thread::sleep(std::time::Duration::from_millis(1));
        });
    }

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

    {
        let mut input_amplitudes = vec![];
        let mut output_amplitudes = vec![];
        system.main_loop(move |_, ui| {
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
