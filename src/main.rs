use clap::Parser;
use clap_num::maybe_hex;
use rust_nextys_monitoring::config::Config;
//use rust_nextys_monitoring::database;
use tokio_modbus::client::rtu;
use tokio_modbus::prelude::Reader;
use tokio_modbus::slave::Slave;
use tokio_serial::SerialStream;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_parser=maybe_hex::<u16>)]
    register: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let _config = Config::load("config.toml").expect("Couldn't expect config");
    let builder = tokio_serial::new("/dev/ttyACM0", 19_200);
    let port = SerialStream::open(&builder).expect("Error opening the serial port");
    let slave = Slave(0x01);
    // let mut ctx = rtu::connect_slave(&port, slave).expect("expected connection");
    let mut ctx = rtu::attach_slave(port, slave);
    let cv = ctx.read_holding_registers(args.register, 1).await?;
    println!("{:?}", cv.unwrap());

    Ok(())
}
