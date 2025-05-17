use serialport::SerialPort;
use tokio_modbus::{Slave, SlaveId};

fn main() {
    let port = serialport::new("/dev/ttyACM0", 19_200)
        .open()
        .expect("Failed to open port");
}
