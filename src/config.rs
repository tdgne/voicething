use clap::Clap;
use getset::{Getters, Setters};

#[derive(Clap, Getters, Clone)]
#[clap(version = "0.0", author = "tdgne")]
pub struct CommandLineOptions {
    #[clap(short, long)]
    #[getset(get = "pub")]
    input_file: Option<String>,
}

impl CommandLineOptions {
    pub fn parse_pub() -> Self {
        Self::parse()
    }
}

#[derive(Clone)]
pub enum Input {
    Default,
    Device(String),
    File(String),
}

#[derive(Clone)]
pub enum Output {
    Default,
    Device(String),
}

// TODO: use wither
#[derive(Getters, Setters, Clone)]
#[getset(get = "pub", set = "pub")]
pub struct Config {
    input: Option<Input>,
    output: Option<Output>,
}

impl Config {
    pub fn new(
        options: CommandLineOptions,
        default_input: Option<Input>,
        default_output: Option<Output>,
    ) -> Self {
        let input = options
            .input_file
            .map(|file| Input::File(file))
            .or(default_input);
        let output = default_output;
        Self { input, output }
    }
}
