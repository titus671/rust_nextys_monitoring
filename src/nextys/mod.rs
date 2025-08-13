use log::error;
use tokio_modbus::client::sync::{Context, rtu};
use tokio_modbus::prelude::SyncReader;
use tokio_modbus::slave::Slave;
mod meters;
mod settings;
use crate::convert_to_signed;
use crate::nextys::meters::Meters;
use crate::nextys::settings::{BatteryType, Settings};
pub struct Nextys {
    ctx: Context,
}
impl Nextys {
    pub fn new() -> Self {
        let port = serialport::new("/dev/ttyACM0", 19_200);
        let slave = Slave(0x01);
        let ctx = rtu::connect_slave(&port, slave).expect("error connecting to slave");
        Nextys { ctx }
    }

    pub fn get_address(&mut self, address: u16, count: u16) -> Vec<u16> {
        match self.ctx.read_holding_registers(address, count) {
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

    pub fn get_meters(&mut self) -> Meters {
        let input_voltage = self.get_input_voltage();
        let input_current = self.get_input_current();
        let output_voltage = self.get_output_voltage();
        let output_current = self.get_output_current();
        let batt_voltage = self.get_batt_voltage();
        let batt_current = self.get_batt_current();
        Meters {
            input_voltage,
            input_current,
            output_voltage,
            output_current,
            batt_voltage,
            batt_current,
        }
    }

    pub fn get_settings(&mut self) -> Settings {
        let batt_type = self.get_batt_type();
        let batt_charge_voltage = self.get_batt_charge_voltage();
        let batt_charge_current = self.get_batt_charge_current();
        let batt_float_voltage = self.get_batt_float_voltage();
        let batt_low_voltage = self.get_batt_low_voltage();
        let batt_deep_discharge = self.get_batt_deep_discharge();
        let batt_max_discharge_current = self.get_batt_max_discharge_current();
        let batt_capacity = self.get_batt_capacity();
        let nominal_output_voltage = self.get_nominal_output_voltage();
        let max_input_current = self.get_max_input_current();
        let max_output_current = self.get_max_output_current();
        Settings {
            batt_type,
            batt_charge_voltage,
            batt_charge_current,
            batt_float_voltage,
            batt_low_voltage,
            batt_deep_discharge,
            batt_max_discharge_current,
            batt_capacity,
            nominal_output_voltage,
            max_input_current,
            max_output_current,
        }
    }

    // Settings
    fn get_batt_type(&mut self) -> BatteryType {
        match self.get_address(0x1010, 1)[0] {
            1 => BatteryType::Lead,
            2 => BatteryType::Nickel,
            3 => BatteryType::Lithium,
            4 => BatteryType::Supercapacitor,
            _ => BatteryType::Unknown,
        }
    }
    fn get_batt_charge_voltage(&mut self) -> f32 {
        self.get_address(0x1011, 1)[0] as f32 / 10.0
    }
    fn get_batt_charge_current(&mut self) -> f32 {
        self.get_address(0x1012, 1)[0] as f32 / 10.0
    }
    fn get_batt_float_voltage(&mut self) -> f32 {
        self.get_address(0x1013, 1)[0] as f32 / 10.0
    }
    fn get_batt_low_voltage(&mut self) -> f32 {
        self.get_address(0x1014, 1)[0] as f32 / 10.0
    }
    fn get_batt_deep_discharge(&mut self) -> f32 {
        self.get_address(0x1015, 1)[0] as f32 / 10.0
    }
    fn get_batt_max_discharge_current(&mut self) -> f32 {
        self.get_address(0x1016, 1)[0] as f32 / 10.0
    }
    fn get_batt_capacity(&mut self) -> f32 {
        self.get_address(0x1017, 1)[0] as f32 / 10.0
    }
    fn get_nominal_output_voltage(&mut self) -> f32 {
        self.get_address(0x1021, 1)[0] as f32 / 10.0
    }
    fn get_max_input_current(&mut self) -> f32 {
        self.get_address(0x1022, 1)[0] as f32 / 10.0
    }
    fn get_max_output_current(&mut self) -> f32 {
        self.get_address(0x1023, 1)[0] as f32 / 10.0
    }
    // Metering
    fn get_input_voltage(&mut self) -> f32 {
        convert_to_signed(self.get_address(0x2000, 1)) as f32 / 10.0
    }
    fn get_input_current(&mut self) -> f32 {
        convert_to_signed(self.get_address(0x2001, 1)) as f32 / 10.0
    }
    fn get_output_voltage(&mut self) -> f32 {
        convert_to_signed(self.get_address(0x2002, 1)) as f32 / 10.0
    }
    fn get_output_current(&mut self) -> f32 {
        convert_to_signed(self.get_address(0x2003, 1)) as f32 / 10.0
    }
    fn get_batt_voltage(&mut self) -> f32 {
        convert_to_signed(self.get_address(0x2004, 1)) as f32 / 10.0
    }
    fn get_batt_current(&mut self) -> f32 {
        convert_to_signed(self.get_address(0x2005, 1)) as f32 / 10.0
    }
}
