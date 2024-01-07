use clap::Parser;

/// Structure representing program arguments.
///
/// # Fields
///
/// * `port` - network port (default = 11111)
/// * `hostname` - ip address (default = "localhost")
///
/// # Example
///
/// ```
/// use clap::Parser;
/// use server::args::Args;
///
/// fn main() {
///     let args = Args::parse();
///
///     println!("{}:{}", args.hostname, args.port);
/// }
/// ```
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long, default_value_t = 11111)]
    pub port: u16,

    #[arg(long, default_value = "localhost")]
    pub hostname: String,
}
