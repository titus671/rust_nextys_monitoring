pub mod config;
pub mod database;
pub mod nextys;

pub fn convert_to_signed(input: Vec<u16>) -> i16 {
    i16::from_be_bytes(input[0].to_be_bytes())
}
