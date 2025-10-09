use std::time::Duration;

use chrono::Utc;
use log::error;
use tokio::time;
use tokio_modbus::client::{Context, rtu};
use tokio_modbus::prelude::Reader;
use tokio_modbus::slave::Slave;
use tokio_serial::SerialStream;
pub mod meters;
pub mod settings;
use crate::convert_to_signed;
use crate::nextys::meters::Meters;
use crate::nextys::settings::{BatteryType, Settings};

pub struct Nextys {
    ctx: Context,
}
impl Nextys {
    pub fn new() -> Self {
        let builder = tokio_serial::new("/dev/ttyACM0", 19_200);
        let port = SerialStream::open(&builder).expect("Error opening the serial port");
        let slave = Slave(0x01);
        let ctx = rtu::attach_slave(port, slave);
        Nextys { ctx }
    }

    pub async fn get_address(&mut self, address: u16, count: u16) -> Vec<u16> {
        match self.ctx.read_holding_registers(address, count).await {
            Ok(data) => match data {
                Ok(data) => data,
                Err(e) => {
                    error!("failed to read address {address}: {:?}", e);
                    vec![0]
                }
            },
            Err(e) => {
                error!("Failed to read address {address}: {:?}", e);
                vec![0]
            }
        }
    }

    pub async fn get_avg_meters(&mut self) -> Meters {
        let mut meters: Vec<Meters> = Vec::new();
        let now = Utc::now().timestamp();

        while Utc::now().timestamp() < now + 10 {
            meters.push(self.get_meters().await);
            time::sleep(Duration::from_secs(1)).await;
        }
        Meters::average(meters)
    }

    pub async fn get_meters(&mut self) -> Meters {
        let input_voltage = self.get_input_voltage().await;
        let input_current = self.get_input_current().await;
        let output_voltage = self.get_output_voltage().await;
        let output_current = self.get_output_current().await;
        let batt_voltage = self.get_batt_voltage().await;
        let batt_current = self.get_batt_current().await;
        let batt_soc = self.get_batt_soc().await;
        let batt_int_resistance = self.get_batt_int_resistance().await;
        Meters {
            input_voltage,
            input_current,
            output_voltage,
            output_current,
            batt_voltage,
            batt_current,
            batt_soc,
            batt_int_resistance,
        }
    }

    pub async fn get_settings(&mut self) -> Settings {
        let batt_type = self.get_batt_type().await;
        let batt_type_int = self.get_batt_type_int().await;
        let batt_charge_voltage = self.get_batt_charge_voltage().await;
        let batt_charge_current = self.get_batt_charge_current().await;
        let batt_float_voltage = self.get_batt_float_voltage().await;
        let batt_low_voltage = self.get_batt_low_voltage().await;
        let batt_deep_discharge_voltage = self.get_batt_deep_discharge_voltage().await;
        let batt_max_discharge_current = self.get_batt_max_discharge_current().await;
        let batt_capacity = self.get_batt_capacity().await;
        let nominal_output_voltage = self.get_nominal_output_voltage().await;
        let max_input_current = self.get_max_input_current().await;
        let max_output_current = self.get_max_output_current().await;
        Settings {
            batt_type,
            batt_type_int,
            batt_charge_voltage,
            batt_charge_current,
            batt_float_voltage,
            batt_low_voltage,
            batt_deep_discharge_voltage,
            batt_max_discharge_current,
            batt_capacity,
            nominal_output_voltage,
            max_input_current,
            max_output_current,
        }
    }

    // Settings
    async fn get_batt_type(&mut self) -> BatteryType {
        match self.get_address(0x1010, 1).await[0] {
            1 => BatteryType::Lead,
            2 => BatteryType::Nickel,
            3 => BatteryType::Lithium,
            4 => BatteryType::Supercapacitor,
            _ => BatteryType::Unknown,
        }
    }
    async fn get_batt_type_int(&mut self) -> i16 {
        self.get_address(0x1010, 1).await[0] as i16
    }
    async fn get_batt_charge_voltage(&mut self) -> f32 {
        self.get_address(0x1011, 1).await[0] as f32 / 10.0
    }
    async fn get_batt_charge_current(&mut self) -> f32 {
        self.get_address(0x1012, 1).await[0] as f32 / 10.0
    }
    async fn get_batt_float_voltage(&mut self) -> f32 {
        self.get_address(0x1013, 1).await[0] as f32 / 10.0
    }
    async fn get_batt_low_voltage(&mut self) -> f32 {
        self.get_address(0x1014, 1).await[0] as f32 / 10.0
    }
    async fn get_batt_deep_discharge_voltage(&mut self) -> f32 {
        self.get_address(0x1015, 1).await[0] as f32 / 10.0
    }
    async fn get_batt_max_discharge_current(&mut self) -> f32 {
        self.get_address(0x1016, 1).await[0] as f32 / 10.0
    }
    async fn get_batt_capacity(&mut self) -> f32 {
        self.get_address(0x1017, 1).await[0] as f32 / 10.0
    }
    async fn get_nominal_output_voltage(&mut self) -> f32 {
        self.get_address(0x1021, 1).await[0] as f32 / 10.0
    }
    async fn get_max_input_current(&mut self) -> f32 {
        self.get_address(0x1022, 1).await[0] as f32 / 10.0
    }
    async fn get_max_output_current(&mut self) -> f32 {
        self.get_address(0x1023, 1).await[0] as f32 / 10.0
    }
    // Metering
    async fn get_input_voltage(&mut self) -> f32 {
        convert_to_signed(self.get_address(0x2000, 1).await) as f32 / 10.0
    }
    async fn get_input_current(&mut self) -> f32 {
        convert_to_signed(self.get_address(0x2001, 1).await) as f32 / 10.0
    }
    async fn get_output_voltage(&mut self) -> f32 {
        convert_to_signed(self.get_address(0x2002, 1).await) as f32 / 10.0
    }
    async fn get_output_current(&mut self) -> f32 {
        convert_to_signed(self.get_address(0x2003, 1).await) as f32 / 10.0
    }
    async fn get_batt_voltage(&mut self) -> f32 {
        convert_to_signed(self.get_address(0x2004, 1).await) as f32 / 10.0
    }
    async fn get_batt_current(&mut self) -> f32 {
        convert_to_signed(self.get_address(0x2005, 1).await) as f32 / 10.0
    }
    async fn get_batt_soc(&mut self) -> f32 {
        convert_to_signed(self.get_address(0x200A, 1).await) as f32 / 10.0
    }
    async fn get_batt_int_resistance(&mut self) -> f32 {
        convert_to_signed(self.get_address(0x2009, 1).await) as f32 / 10.0
    }
}
