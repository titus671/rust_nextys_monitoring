use clap::Parser;
use clap_num::maybe_hex;
use rust_nextys_monitoring::database;
// use tokio_modbus::client::sync::rtu;
// use tokio_modbus::prelude::SyncReader;
// use tokio_modbus::slave::Slave;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_parser=maybe_hex::<u16>)]
    register: u16,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // let args = Args::parse();
    // let port = serialport::new("/dev/ttyACM0", 19_200);
    // let slave = Slave(0x01);
    // let mut ctx = rtu::connect_slave(&port, slave).expect("expected connection");
    // let cv = ctx.read_holding_registers(args.register, 1);
    // println!("{:?}", cv);
    database::initialise_database().await?;
    Ok(())
}
