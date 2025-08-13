use serde_derive::Deserialize;
use std::{fs, net::Ipv4Addr};
use toml;

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub timescaledb: TimescaleDB,
    pub device_id: Option<i64>,
    pub ip_address: Option<Ipv4Addr>,
    pub sys_name: String,
    pub location: String,
    pub low_batt_threshold: f32,
    pub ac_down_threshold: i64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TimescaleDB {
    pub timescaledb_host: Ipv4Addr,
    pub timescaledb_port: i64,
    pub timescaledb_user: String,
    pub timescaledb_pass: String,
    pub timescaledb_db: String,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path).expect(
            format!(
                "should have been able to find file, make sure the provided path: '{}'  exists",
                path
            )
            .as_str(),
        );
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
