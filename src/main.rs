use clap::Clap;
use rodio;
use std::io::BufReader;
use std::thread;
use std::time;

mod stream;
use stream::{
    MultipleOutputNode, Multiplexer, Node, PlaybackSink, PsolaNode, SingleOutputNode, StaticSource,
    WriteSink, RecordingSource
};

mod common;

mod gui;

#[derive(Clap, Clone)]
#[clap(version = "1.0", author = "tdgne")]
struct Opts {
    #[clap(short, long)]
    input_file: Option<String>,
    #[clap(short, long)]
    output_file: Option<String>,
}

fn main() {
    let opts = Opts::parse();

    {
        let opts = opts.clone();
        thread::spawn(move || {
            let device = rodio::default_output_device().unwrap();
            let rsink = rodio::Sink::new(&device);
            let mut src: Box<dyn SingleOutputNode<f32>> = if let Some(input_file) = opts.input_file {
                let file = std::fs::File::open(input_file).unwrap();
                Box::new(StaticSource::new(BufReader::new(file), 2048).unwrap())
            } else {
                Box::new(RecordingSource::new(1024))
            };
            let mut psola = PsolaNode::new(src.output(), 1.5);
            let mut m = Multiplexer::new(psola.output());
            let psink = PlaybackSink::new(m.new_output(), rsink);
            if let Some(output_file) = opts.output_file {
                let wsink = WriteSink::new(m.new_output(), output_file);
                thread::spawn(move || {
                    wsink.run(time::Duration::from_millis(100));
                });
            }
            thread::spawn(move || {
                src.run();
            });
            thread::spawn(move || {
                m.run();
            });
            let playback_thread = thread::spawn(move || {
                psink.start_playback();
            });
            thread::spawn(move || {
                psola.run();
            });
            // src.play_all(false);
            playback_thread.join().unwrap();
        });
    }

    gui::main_loop();
}
