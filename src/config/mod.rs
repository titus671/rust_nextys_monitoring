use anyhow::{Context, Result};
use serde::Serialize;
use serde_derive::Deserialize;
use std::{fs, net::IpAddr};
use toml;

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct Config {
    pub timescaledb: TimescaleDB,
    pub device_id: Option<i32>,
    pub ip_address: IpAddr,
    pub sys_name: String,
    pub location: String,
    pub low_batt_threshold: f32,
    pub ac_down_threshold: f32,
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct TimescaleDB {
    pub timescaledb_host: IpAddr,
    pub timescaledb_port: i64,
    pub timescaledb_user: String,
    pub timescaledb_pass: String,
    pub timescaledb_db: String,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, toml::de::Error> {
        let content = fs::read_to_string(path).expect(
            format!(
                "should have been able to find file, make sure the provided path: '{}'  exists",
                path
            )
            .as_str(),
        );
        let config_res: Result<Config, toml::de::Error> = toml::from_str(&content);
        let config = match config_res {
            Ok(config) => Ok(config),
            Err(error) => Err(error),
        };
        config
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let toml_str =
            toml::to_string_pretty(self).context("Failed to serialize config to TOML")?;
        fs::write(path, toml_str).context("Failed to write to save file")?;
        Ok(())
    }
}
