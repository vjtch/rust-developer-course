use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long, default_value_t = 11111)]
    pub port: u16,

    #[arg(long, default_value = "localhost")]
    pub hostname: String,
}
