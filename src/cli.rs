
use clap::{Parser, ValueEnum};



/// A simple local file sharing app.
/// 
///  Created at July 2025.
#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
pub struct Cli {

    /// The directory to save uploaded files
    #[arg(short, long, default_value_t = String::from("uploads"))]
    pub dir : String,

    /// The port to listen on, default is 8080
    #[arg(long, default_value_t = 8080, value_parser = port_in_range)]
    pub port : u16,

    /// Whether to create parent dirs if not exists
    #[arg(short = 'p')]
    pub create_parent_dirs : bool,

    /// Log level
    #[arg(short = 'l', long = "level")]
    pub log_level : Option<CliLogLevel>,
}


/// Custom validator: Port must be between 1024 and 65535
fn port_in_range(s: &str) -> Result<u16, String> {
    let port: u16 = s.parse().map_err(|_| format!("`{}` is not a valid port number", s))?;
    if port < 1024 {
        Err(format!("Port {} is too low. Use a port >= 1024", port))
    } else {
        Ok(port)
    }
}


/// Possible log levels (case-insensitive)
#[derive(Copy, Clone, Debug, ValueEnum)]
#[clap(rename_all = "lowercase")]
pub enum CliLogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
/// `LogLevel` → `log::LevelFilter` dönüşümü
impl From<CliLogLevel> for log::LevelFilter {
    fn from(level: CliLogLevel) -> Self {
        match level {
            CliLogLevel::Off => log::LevelFilter::Off,
            CliLogLevel::Error => log::LevelFilter::Error,
            CliLogLevel::Warn => log::LevelFilter::Warn,
            CliLogLevel::Info => log::LevelFilter::Info,
            CliLogLevel::Debug => log::LevelFilter::Debug,
            CliLogLevel::Trace => log::LevelFilter::Trace,
        }
    }
}
