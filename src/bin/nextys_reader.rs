use clap::{Parser, Subcommand};
use clap_num::maybe_hex;
use env_logger::Env;
use rust_nextys_monitoring::{config::Config, nextys::Nextys};

#[derive(Parser)]
#[command(name = "Nextys Reader")]
#[command(version = "0.1")]
#[command(about = "Read meaters from Nextys DCW20")]
struct Cli {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    /// Read a single meter
    ReadAddress {
        /// Meter address in hexadecimal ie 0x1018
        #[arg(short, long, value_parser=maybe_hex::<u16>)]
        address: u16,

        /// Loop read address
        #[arg(short, long)]
        to_loop: bool,

        /// How many registers to aggregate together
        #[arg(short, long, default_value = "1")]
        count: u16,
    },
    /// Read all meters
    ReadMeters {
        /// Loop reading meters
        #[arg(short, long)]
        to_loop: bool,
    },
    /// Read Settings
    ReadSettings {},
    /// Show Config
    ShowConfig {
        /// Config path
        #[arg(short, long, default_value = "config.toml")]
        path: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let cli = Cli::parse();
    let mut nextys = Nextys::new();
    match cli.action {
        Action::ReadAddress {
            address,
            to_loop,
            count,
        } => {
            if to_loop {
                loop {
                    let reading = nextys.get_address(address, count);
                    println!("{:?}", reading);
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                }
            } else {
                let reading = nextys.get_address(address, count);
                println!("{:?}", reading);
            }
        }
        Action::ReadMeters { to_loop } => {
            if to_loop {
                loop {
                    let meters = nextys.get_meters();
                    println!("{:#?}", meters);
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                }
            } else {
                let meters = nextys.get_meters();
                println!("{:#?}", meters);
            }
        }
        Action::ReadSettings {} => {
            let settings = nextys.get_settings();
            println!("{:#?}", settings);
        }
        Action::ShowConfig { path } => {
            let config = Config::load(path.as_str()).unwrap();
            println!("{:#?}", config)
        }
    }
    Ok(())
}
