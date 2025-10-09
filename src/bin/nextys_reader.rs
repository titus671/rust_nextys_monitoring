use clap::{Parser, Subcommand};
use clap_num::maybe_hex;
use env_logger::Env;
use log::info;
use rust_nextys_monitoring::{config::Config, database, nextys::Nextys};

const DEFAULT_CONFIG_PATH: &str = "config.toml";

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
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config_path: String,
    },
    /// Initialize Database and device
    InitializeDevice {
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config_path: String,
    },
    /// Upload Settings
    UploadSettings {
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        /// Config path
        config_path: String,
    },
    /// Upload Meters
    UploadMeters {
        /// Config path
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config_path: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("error")).init();
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
                    let reading = nextys.get_address(address, count).await;
                    println!("{:?}", reading);
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                }
            } else {
                let reading = nextys.get_address(address, count).await;
                println!("{:?}", reading);
            }
        }
        Action::ReadMeters { to_loop } => {
            if to_loop {
                loop {
                    let meters = nextys.get_meters().await;
                    println!("{:#?}", meters);
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                }
            } else {
                let meters = nextys.get_avg_meters().await;
                println!("{:#?}", meters);
            }
        }
        Action::ReadSettings {} => {
            let settings = nextys.get_settings().await;
            println!("{:#?}", settings);
        }
        Action::ShowConfig { config_path } => {
            let mut config = Config::load(config_path.as_str()).unwrap();
            println!("{:#?}", config);
            config.device_id = Some(20);
            config.save(config_path.as_str()).unwrap();
        }
        Action::InitializeDevice { config_path } => {
            let mut config = Config::load(config_path.as_str()).unwrap();
            let pool = database::initialize_connection(config.clone()).await?;
            let _ = database::get_id(pool, &mut config).await?;
            match config.save(config_path.as_str()) {
                Ok(_) => println!("Succesfully wrote {:#?} to {}", config, config_path),
                Err(e) => panic!("Error 1: {e}"),
            };
            println!("{:#?}", config);
        }
        Action::UploadSettings { config_path } => {
            let config = Config::load(config_path.as_str()).unwrap();
            let pool = database::initialize_connection(config.clone()).await?;
            let settings = nextys.get_settings().await;
            match database::upload_settings(pool, &config, &settings).await {
                Ok(_) => info!("Uploaded settings"),
                Err(e) => panic!("Error 2:{e}"),
            };
        }
        Action::UploadMeters { config_path } => {
            let config = Config::load(config_path.as_str()).unwrap();
            let pool = database::initialize_connection(config.clone()).await?;
            loop {
                let meters = nextys.get_avg_meters().await;
                let result = database::upload_metrics(&pool, &config, &meters).await;
                match result {
                    Ok(_) => info!("Uploaded Metrics"),
                    Err(e) => {
                        pool.close().await;
                        panic!("Error 3:{e}")
                    }
                }
            }
        }
    }
    Ok(())
}
