#[derive(Debug)]
pub struct Settings {
    pub batt_type: BatteryType,
    pub batt_type_int: i16,
    pub batt_charge_voltage: f32,
    pub batt_charge_current: f32,
    pub batt_float_voltage: f32,
    pub batt_low_voltage: f32,
    pub batt_deep_discharge_voltage: f32,
    pub batt_max_discharge_current: f32,
    pub batt_capacity: f32,
    pub nominal_output_voltage: f32,
    pub max_input_current: f32,
    pub max_output_current: f32,
}

#[derive(Debug)]
pub enum BatteryType {
    Lead,
    Nickel,
    Lithium,
    Supercapacitor,
    Unknown,
}
