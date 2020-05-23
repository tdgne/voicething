use clap::Clap;
use getset::Getters;

#[derive(Clap, Getters, Clone)]
#[clap(version = "0.0", author = "tdgne")]
pub struct Options {
    #[clap(short, long)]
    #[getset(get = "pub")]
    input_file: Option<String>,
    #[clap(short, long)]
    #[getset(get = "pub")]
    output_file: Option<String>,
}

impl Options {
    pub fn parse_pub() -> Self {
        Self::parse()
    }
}
