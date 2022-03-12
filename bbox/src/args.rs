use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct Args {
    /// Server use port
    #[clap(short, long, default_value_t = 3000)]
    pub port: u16,

    /// Read config path
    #[clap(short, long, default_value("./config.json"))]
    pub config: String,

    /// Scan directory path
    #[clap(short, long, default_value("./static"))]
    pub scan: String,

    /// Log output path
    #[clap(short, long, default_value("./logs"))]
    pub log: String,
}