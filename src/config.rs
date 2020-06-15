use clap::Clap;
use getset::Getters;

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
