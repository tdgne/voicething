use clap::Clap;
use rodio;
use std::io::BufReader;
use std::thread;

mod stream;
use stream::{PlaybackSink, StaticSource};

mod common;

/*
use gio::prelude::*;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button};
*/

#[derive(Clap, Clone)]
#[clap(version = "1.0", author = "tdgne")]
struct Opts {
    input_file: String,
}

fn main() {
    let opts = Opts::parse();

    let handle = {
        let opts = opts.clone();
        thread::spawn(|| {
            let device = rodio::default_output_device().unwrap();
            let rsink = rodio::Sink::new(&device);
            let file = std::fs::File::open(opts.input_file).unwrap();
            let mut src = StaticSource::new(BufReader::new(file), 512).unwrap();
            let rx = src.new_receiver();
            let playback_thread = thread::spawn(move || {
                let sink = PlaybackSink::new(rx, rsink);
                sink.start_playback();
            });
            src.play_all();
            playback_thread.join().unwrap();
        })
    };

    handle.join().unwrap();

    /*
    let application =
        Application::new(None, Default::default()).expect("failed to initialize GTK application");

    application.connect_activate(|app| {
        let window = ApplicationWindow::new(app);
        window.set_title("Voice Converter");
        window.set_default_size(350, 70);

        let button = Button::new_with_label("Click me!");
        button.connect_clicked(|_| {
            println!("Clicked!");
        });
        window.add(&button);

        window.show_all();
    });

    application.run(&[]);
    */
}
